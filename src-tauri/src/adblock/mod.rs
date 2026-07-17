pub mod engine;
pub mod rules;

pub use engine::{AdblockEngine, AdblockExport};

use std::sync::Arc;
use tauri::{AppHandle, Manager};

pub const BUILTIN_RULES: &str = include_str!("../../assets/easylist-mini.txt");

pub struct AdblockState {
    pub engine: Arc<AdblockEngine>,
    pub enabled: bool,
}

impl AdblockState {
    pub fn new(enabled: bool, custom_rules_text: &str) -> Self {
        Self {
            engine: Arc::new(AdblockEngine::from_rules_text(BUILTIN_RULES, custom_rules_text)),
            enabled,
        }
    }
}

pub fn update_tray_block_count(app: &AppHandle, count: u32) {
    let tooltip = if count == 0 {
        "Pake Plus".to_string()
    } else {
        format!("已拦截 {count} 个请求")
    };

    if let Some(tray) = app.tray_by_id("pake-tray") {
        let _ = tray.set_tooltip(Some(&tooltip));
        #[cfg(target_os = "macos")]
        {
            let _ = tray.set_title(Some(&count.to_string()));
        }
    }
}

pub fn reset_page_block_count(app: &AppHandle) {
    if let Some(state) = app.try_state::<AdblockState>() {
        if state.enabled {
            state.engine.reset_page_count();
            update_tray_block_count(app, 0);
        }
    }
}
