pub mod commands;
pub mod diagnostics;
pub mod health;
pub mod io;
pub mod traits;
pub mod types;

use tauri::Manager;

// Re-export commonly used items
pub use commands::{
    copy_diagnostics_report, export_data, get_diagnostics, get_module_stats, get_settings,
    import_data, list_backups, preview_import, reset_settings, rollback_settings, save_settings,
    validate_settings,
};
pub use diagnostics::get_diagnostics_report;
pub use health::run_health_check;
pub use io::load_settings;
pub use types::AppSettings;

// Window helper
pub fn open_settings_panel(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("pake") {
        let _ = window.eval("window.__pakeOpenSettings()");
    }
}

// Convenience function for other modules
pub fn get_settings_inner(app: &tauri::AppHandle) -> AppSettings {
    load_settings(app)
}

#[cfg(test)]
mod tests;
