use super::filter::{should_record, FilterConfig};
use super::settings::ClipboardSettings;
use super::source::current_application;
use super::store::ClipboardStore;
use clipboard_rs::{
    Clipboard, ClipboardContext, ClipboardHandler, ClipboardWatcher, ClipboardWatcherContext,
    WatcherShutdown,
};
#[cfg(target_os = "linux")]
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[cfg(target_os = "linux")]
const POLL_INTERVAL_MS: u64 = 300;

pub type ExpectedHash = Arc<Mutex<Option<String>>>;

enum MonitorStop {
    Native(WatcherShutdown),
    #[cfg(target_os = "linux")]
    Poll(Arc<AtomicBool>),
}

pub struct ClipboardMonitor {
    stop: Option<MonitorStop>,
    thread: Option<JoinHandle<()>>,
}

impl ClipboardMonitor {
    pub fn stop(mut self) {
        if let Some(stop) = self.stop.take() {
            match stop {
                MonitorStop::Native(shutdown) => shutdown.stop(),
                #[cfg(target_os = "linux")]
                MonitorStop::Poll(flag) => flag.store(true, Ordering::Release),
            }
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for ClipboardMonitor {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            match stop {
                MonitorStop::Native(shutdown) => shutdown.stop(),
                #[cfg(target_os = "linux")]
                MonitorStop::Poll(flag) => flag.store(true, Ordering::Release),
            }
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

struct HistoryHandler {
    clipboard: ClipboardContext,
    last_hash: Option<String>,
    store: Arc<ClipboardStore>,
    settings: Arc<RwLock<ClipboardSettings>>,
    expected_hash: ExpectedHash,
}

impl ClipboardHandler for HistoryHandler {
    fn on_clipboard_change(&mut self) {
        if let Ok(text) = self.clipboard.get_text() {
            let hash = ClipboardStore::hash_content(&text);
            if consume_expected_hash(&self.expected_hash, &hash) {
                self.last_hash = Some(hash);
                return;
            }
            if self.last_hash.as_deref() == Some(&hash) {
                return;
            }
            self.last_hash = Some(hash);
            let source_app = current_application();
            process_text(
                &text,
                source_app.as_deref(),
                &self.store,
                &self.settings,
                &self.expected_hash,
            );
        }
    }
}

pub fn start_monitor(
    store: Arc<ClipboardStore>,
    settings: Arc<RwLock<ClipboardSettings>>,
    expected_hash: ExpectedHash,
) -> Result<ClipboardMonitor, String> {
    if pure_wayland_session() {
        return start_wayland_poll(store, settings, expected_hash);
    }

    let clipboard = ClipboardContext::new().map_err(|error| error.to_string())?;
    let last_hash = clipboard
        .get_text()
        .ok()
        .map(|text| ClipboardStore::hash_content(&text));
    let mut watcher = ClipboardWatcherContext::new().map_err(|error| error.to_string())?;
    watcher.add_handler(HistoryHandler {
        clipboard,
        last_hash,
        store,
        settings,
        expected_hash,
    });
    let shutdown = watcher.get_shutdown_channel();
    let thread = thread::spawn(move || watcher.start_watch());

    Ok(ClipboardMonitor {
        stop: Some(MonitorStop::Native(shutdown)),
        thread: Some(thread),
    })
}

fn process_text(
    text: &str,
    source_app: Option<&str>,
    store: &ClipboardStore,
    settings: &RwLock<ClipboardSettings>,
    expected_hash: &Mutex<Option<String>>,
) {
    let hash = ClipboardStore::hash_content(text);
    if consume_expected_hash(expected_hash, &hash) {
        return;
    }

    let Ok(settings) = settings.read() else {
        return;
    };
    if !settings.enabled {
        return;
    }
    let filter = FilterConfig {
        ignore_short_text: settings.ignore_short_text,
        min_length: settings.min_length,
        max_length: settings.max_length,
        ignore_password_like: settings.ignore_password_like,
        ignore_credit_card_like: settings.ignore_credit_card_like,
        ignored_apps: settings.ignored_apps.clone(),
    };
    if should_record(text, source_app, &filter).is_err() {
        return;
    }
    if let Err(error) = store.insert_with_source(text, source_app) {
        eprintln!("[Pake] Failed to store clipboard entry: {error}");
    }
}

fn consume_expected_hash(expected_hash: &Mutex<Option<String>>, actual: &str) -> bool {
    let Ok(mut expected) = expected_hash.lock() else {
        return false;
    };
    if expected.as_deref() == Some(actual) {
        expected.take();
        true
    } else {
        false
    }
}

#[cfg(target_os = "linux")]
fn pure_wayland_session() -> bool {
    std::env::var("WAYLAND_DISPLAY")
        .ok()
        .is_some_and(|value| !value.trim().is_empty())
        && std::env::var("DISPLAY")
            .ok()
            .is_none_or(|value| value.trim().is_empty())
}

#[cfg(not(target_os = "linux"))]
fn pure_wayland_session() -> bool {
    false
}

fn start_wayland_poll(
    store: Arc<ClipboardStore>,
    settings: Arc<RwLock<ClipboardSettings>>,
    expected_hash: ExpectedHash,
) -> Result<ClipboardMonitor, String> {
    #[cfg(target_os = "linux")]
    {
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let thread = thread::spawn(move || {
            let mut last_hash = read_wayland_text()
                .ok()
                .map(|text| ClipboardStore::hash_content(&text));
            while !thread_stop.load(Ordering::Acquire) {
                thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
                let Ok(text) = read_wayland_text() else {
                    continue;
                };
                let hash = ClipboardStore::hash_content(&text);
                if consume_expected_hash(&expected_hash, &hash) {
                    last_hash = Some(hash);
                    continue;
                }
                if last_hash.as_deref() == Some(&hash) {
                    continue;
                }
                last_hash = Some(hash);
                let source_app = current_application();
                process_text(
                    &text,
                    source_app.as_deref(),
                    &store,
                    &settings,
                    &expected_hash,
                );
            }
        });
        Ok(ClipboardMonitor {
            stop: Some(MonitorStop::Poll(stop)),
            thread: Some(thread),
        })
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (store, settings, expected_hash);
        Err("Wayland clipboard polling is only available on Linux".to_string())
    }
}

#[cfg(target_os = "linux")]
fn read_wayland_text() -> Result<String, String> {
    use std::io::Read;
    use wl_clipboard_rs::paste::{get_contents, ClipboardType, MimeType, Seat};

    let (mut pipe, _) = get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Text)
        .map_err(|error| error.to_string())?;
    let mut text = String::new();
    pipe.read_to_string(&mut text)
        .map_err(|error| error.to_string())?;
    Ok(text)
}

pub fn write_text_to_clipboard(text: &str, expected_hash: &ExpectedHash) -> Result<(), String> {
    let hash = ClipboardStore::hash_content(text);
    if let Ok(mut expected) = expected_hash.lock() {
        *expected = Some(hash.clone());
    }

    let result = write_system_text(text);
    if result.is_err() {
        if let Ok(mut expected) = expected_hash.lock() {
            expected.take();
        }
    } else {
        let expected_hash = Arc::clone(expected_hash);
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(1));
            if let Ok(mut expected) = expected_hash.lock() {
                if expected.as_deref() == Some(&hash) {
                    expected.take();
                }
            }
        });
    }
    result
}

fn write_system_text(text: &str) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    if pure_wayland_session() {
        use wl_clipboard_rs::copy::{MimeType, Options, Source};
        return Options::new()
            .copy(Source::Bytes(text.as_bytes().into()), MimeType::Text)
            .map_err(|error| error.to_string());
    }

    let clipboard = ClipboardContext::new().map_err(|error| error.to_string())?;
    clipboard
        .set_text(text.to_string())
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consumes_only_the_expected_clipboard_hash() {
        let expected = Mutex::new(Some("expected".to_string()));
        assert!(!consume_expected_hash(&expected, "different"));
        assert_eq!(expected.lock().unwrap().as_deref(), Some("expected"));
        assert!(consume_expected_hash(&expected, "expected"));
        assert!(expected.lock().unwrap().is_none());
    }
}
