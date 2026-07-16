use super::store::ClipboardStore;
use std::sync::{
    mpsc::{self, RecvTimeoutError, Sender},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const CLEANUP_INTERVAL: Duration = Duration::from_secs(60 * 60);

pub struct ClipboardCleanup {
    stop_tx: Sender<()>,
    handle: Option<JoinHandle<()>>,
}

impl ClipboardCleanup {
    pub fn stop(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        let _ = self.stop_tx.send(());
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for ClipboardCleanup {
    fn drop(&mut self) {
        self.shutdown();
    }
}

pub fn start_cleanup(store: Arc<ClipboardStore>) -> Result<ClipboardCleanup, String> {
    start_cleanup_loop(CLEANUP_INTERVAL, move || {
        if let Err(error) = store.cleanup() {
            eprintln!("[Pake] Failed to clean clipboard history: {error}");
        }
    })
}

fn start_cleanup_loop<F>(interval: Duration, mut cleanup: F) -> Result<ClipboardCleanup, String>
where
    F: FnMut() + Send + 'static,
{
    let (stop_tx, stop_rx) = mpsc::channel();
    let handle = thread::Builder::new()
        .name("pake-clipboard-cleanup".to_string())
        .spawn(move || loop {
            match stop_rx.recv_timeout(interval) {
                Ok(()) | Err(RecvTimeoutError::Disconnected) => break,
                Err(RecvTimeoutError::Timeout) => cleanup(),
            }
        })
        .map_err(|error| error.to_string())?;

    Ok(ClipboardCleanup {
        stop_tx,
        handle: Some(handle),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_cleanup_without_clipboard_activity() {
        let (called_tx, called_rx) = mpsc::channel();
        let worker = start_cleanup_loop(Duration::from_millis(5), move || {
            let _ = called_tx.send(());
        })
        .unwrap();

        called_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        worker.stop();
    }
}
