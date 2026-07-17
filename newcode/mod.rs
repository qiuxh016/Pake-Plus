pub mod commands;
pub mod engine;

use engine::CacheEngine;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

/// Managed state stored inside Tauri.
pub struct CacheState {
    pub engine: Arc<CacheEngine>,
    pub enabled: bool,
}

impl CacheState {
    pub fn new(cache_dir: std::path::PathBuf, enabled: bool, max_size_mb: u32) -> Self {
        Self {
            engine: Arc::new(CacheEngine::new(cache_dir, max_size_mb)),
            enabled,
        }
    }
}

/// Update the system tray tooltip to reflect the current cache stats.
pub fn update_tray_cache_info(app: &AppHandle) {
    if let Some(state) = app.try_state::<CacheState>() {
        if state.enabled {
            let stats = state.engine.stats();
            let mb = stats.total_size as f64 / (1024.0 * 1024.0);
            let max_mb = stats.max_size as f64 / (1024.0 * 1024.0);
            let tooltip = format!("Cache: {:.0}/{:.0} MB, {} files", mb, max_mb, stats.file_count);
            if let Some(tray) = app.tray_by_id("pake-tray") {
                let _ = tray.set_tooltip(Some(&tooltip));
            }
        }
    }
}

/// Sync runtime cache engine config with the persisted settings.
/// Called from `save_settings` when cache settings change.
pub fn sync_cache_config(app: &AppHandle, enabled: bool, max_size_mb: u32) {
    if let Some(state) = app.try_state::<CacheState>() {
        // Resize the cache if the max size changed.
        state.engine.resize(max_size_mb);
        eprintln!(
            "[Pake] Cache config synced: enabled={}, max_size={}MB",
            enabled, max_size_mb
        );
    }
}
