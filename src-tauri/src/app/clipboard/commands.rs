use super::cleanup::{start_cleanup, ClipboardCleanup};
use super::monitor::{start_monitor, ClipboardMonitor, ExpectedHash};
use super::panel;
use super::settings::{settings_path, ClipboardSettings};
use super::store::{ClipboardEntry, ClipboardStats, ClipboardStore};
use crate::app::setup::update_clipboard_tray_state;
use crate::util::get_data_dir;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use tauri::{command, AppHandle, State};

pub struct ClipboardState {
    store: RwLock<Option<Arc<ClipboardStore>>>,
    settings: Arc<RwLock<ClipboardSettings>>,
    settings_file: PathBuf,
    data_dir: PathBuf,
    monitor: Mutex<Option<ClipboardMonitor>>,
    cleanup: Mutex<Option<ClipboardCleanup>>,
    expected_hash: ExpectedHash,
    build_enabled: bool,
}

impl ClipboardState {
    pub fn is_enabled(&self) -> bool {
        self.settings
            .read()
            .map(|settings| settings.enabled)
            .unwrap_or(false)
    }

    fn store(&self) -> Result<Arc<ClipboardStore>, String> {
        self.store
            .read()
            .map_err(|_| "Clipboard store lock poisoned".to_string())?
            .clone()
            .ok_or_else(|| "Clipboard history is disabled".to_string())
    }

    fn ensure_store(&self, settings: &ClipboardSettings) -> Result<Arc<ClipboardStore>, String> {
        if let Some(store) = self
            .store
            .read()
            .map_err(|_| "Clipboard store lock poisoned".to_string())?
            .clone()
        {
            store
                .set_limits(settings.max_items, settings.retention_days)
                .map_err(|error| error.to_string())?;
            return Ok(store);
        }

        let store = Arc::new(
            ClipboardStore::open(&self.data_dir, settings.max_items, settings.retention_days)
                .map_err(|error| error.to_string())?,
        );
        *self
            .store
            .write()
            .map_err(|_| "Clipboard store lock poisoned".to_string())? = Some(Arc::clone(&store));
        Ok(store)
    }

    fn stop_monitor(&self) {
        if let Ok(mut monitor) = self.monitor.lock() {
            if let Some(monitor) = monitor.take() {
                monitor.stop();
            }
        }
    }

    fn start_monitor(&self) -> Result<(), String> {
        if !self.is_enabled() {
            return Ok(());
        }
        let settings = self
            .settings
            .read()
            .map_err(|_| "Clipboard settings lock poisoned".to_string())?
            .clone();
        let store = self.ensure_store(&settings)?;
        let monitor = start_monitor(
            store,
            Arc::clone(&self.settings),
            Arc::clone(&self.expected_hash),
        )?;
        *self
            .monitor
            .lock()
            .map_err(|_| "Clipboard monitor lock poisoned".to_string())? = Some(monitor);
        Ok(())
    }

    fn stop_cleanup(&self) {
        if let Ok(mut cleanup) = self.cleanup.lock() {
            if let Some(cleanup) = cleanup.take() {
                cleanup.stop();
            }
        }
    }

    fn start_cleanup(&self) -> Result<(), String> {
        if !self.is_enabled() {
            return Ok(());
        }
        let cleanup = start_cleanup(self.store()?)?;
        *self
            .cleanup
            .lock()
            .map_err(|_| "Clipboard cleanup lock poisoned".to_string())? = Some(cleanup);
        Ok(())
    }

    fn update_settings(&self, settings: ClipboardSettings) -> Result<ClipboardSettings, String> {
        let settings = settings.normalized(self.build_enabled);
        if settings.enabled {
            self.ensure_store(&settings)?;
        }
        settings
            .save(&self.settings_file)
            .map_err(|error| error.to_string())?;

        self.stop_monitor();
        self.stop_cleanup();
        *self
            .settings
            .write()
            .map_err(|_| "Clipboard settings lock poisoned".to_string())? = settings.clone();
        if !settings.enabled {
            *self
                .store
                .write()
                .map_err(|_| "Clipboard store lock poisoned".to_string())? = None;
        }
        self.start_monitor()?;
        self.start_cleanup()?;
        Ok(settings)
    }
}

impl Drop for ClipboardState {
    fn drop(&mut self) {
        if let Ok(monitor) = self.monitor.get_mut() {
            if let Some(monitor) = monitor.take() {
                monitor.stop();
            }
        }
        if let Ok(cleanup) = self.cleanup.get_mut() {
            if let Some(cleanup) = cleanup.take() {
                cleanup.stop();
            }
        }
    }
}

#[command]
pub fn clipboard_list(
    state: State<'_, ClipboardState>,
    limit: Option<i64>,
) -> Result<Vec<ClipboardEntry>, String> {
    state
        .store()?
        .list(limit.unwrap_or(100).clamp(1, 500))
        .map_err(|error| error.to_string())
}

#[command]
pub fn clipboard_search(
    state: State<'_, ClipboardState>,
    query: String,
    limit: Option<i64>,
) -> Result<Vec<ClipboardEntry>, String> {
    state
        .store()?
        .search(&query, limit.unwrap_or(100).clamp(1, 500))
        .map_err(|error| error.to_string())
}

#[command]
pub fn clipboard_copy_item(
    app: AppHandle,
    state: State<'_, ClipboardState>,
    id: i64,
) -> Result<(), String> {
    panel::copy_item(&app, &state.store()?, &state.expected_hash, id)
}

#[command]
pub fn clipboard_delete_item(state: State<'_, ClipboardState>, id: i64) -> Result<bool, String> {
    state.store()?.delete(id).map_err(|error| error.to_string())
}

#[command]
pub fn clipboard_clear_all(state: State<'_, ClipboardState>) -> Result<(), String> {
    state
        .store()?
        .clear_all()
        .map_err(|error| error.to_string())
}

#[command]
pub fn clipboard_stats(state: State<'_, ClipboardState>) -> Result<ClipboardStats, String> {
    eprintln!("[Pake] clipboard_stats called");
    state.store()?.stats().map_err(|error| error.to_string())
}

#[command]
pub fn clipboard_hide_panel(app: AppHandle) -> Result<(), String> {
    panel::hide_panel(&app);
    Ok(())
}

#[command]
pub fn clipboard_show_panel(app: AppHandle, query: Option<String>) -> Result<(), String> {
    eprintln!("[Pake] clipboard_show_panel called, query={:?}", query);
    match panel::show_panel(&app, query.as_deref()) {
        Ok(()) => {
            eprintln!("[Pake] clipboard_show_panel OK");
            Ok(())
        }
        Err(e) => {
            eprintln!("[Pake] clipboard_show_panel error: {e}");
            Err(e.to_string())
        }
    }
}

#[command]
pub fn clipboard_get_settings(
    state: State<'_, ClipboardState>,
) -> Result<ClipboardSettings, String> {
    state
        .settings
        .read()
        .map(|settings| settings.clone())
        .map_err(|_| "Clipboard settings lock poisoned".to_string())
}

#[command]
pub fn clipboard_update_settings(
    app: AppHandle,
    state: State<'_, ClipboardState>,
    settings: ClipboardSettings,
) -> Result<ClipboardSettings, String> {
    let settings = state.update_settings(settings)?;
    update_clipboard_tray_state(&app, settings.enabled);
    Ok(settings)
}

pub fn init_clipboard_state(
    app: &AppHandle,
    clipboard: bool,
    clipboard_max: u32,
    package_name: String,
) -> Result<ClipboardState, String> {
    let data_dir = get_data_dir(app, package_name).map_err(|error| error.to_string())?;
    let settings_file = settings_path(&data_dir);
    let settings = ClipboardSettings::load(&settings_file, clipboard, clipboard_max);
    settings
        .save(&settings_file)
        .map_err(|error| error.to_string())?;

    let store = if settings.enabled {
        Some(Arc::new(
            ClipboardStore::open(&data_dir, settings.max_items, settings.retention_days)
                .map_err(|error| error.to_string())?,
        ))
    } else {
        None
    };
    let state = ClipboardState {
        store: RwLock::new(store),
        settings: Arc::new(RwLock::new(settings)),
        settings_file,
        data_dir,
        monitor: Mutex::new(None),
        cleanup: Mutex::new(None),
        expected_hash: Arc::new(Mutex::new(None)),
        build_enabled: clipboard,
    };
    state.start_monitor()?;
    state.start_cleanup()?;
    Ok(state)
}
