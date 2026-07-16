use super::types::AppSettings;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub const MAX_BACKUPS: u32 = 5;

pub fn settings_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("pake-settings.json")
}

pub fn ensure_data_dir(app: &AppHandle) {
    if let Ok(dir) = app.path().app_data_dir() {
        let _ = fs::create_dir_all(&dir);
    }
}

pub fn load_settings(app: &AppHandle) -> AppSettings {
    let path = settings_path(app);
    if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default()
    } else {
        let defaults = AppSettings::default();
        ensure_data_dir(app);
        if let Ok(json) = serde_json::to_string_pretty(&defaults) {
            let _ = fs::write(&path, &json);
        }
        defaults
    }
}

pub fn write_settings(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    ensure_data_dir(app);
    let path = settings_path(app);

    // Rotate backups before overwriting
    if path.exists() {
        for i in (1..MAX_BACKUPS).rev() {
            let older = path.with_extension(format!("v{}.json", i));
            let newer = path.with_extension(format!("v{}.json", i + 1));
            if older.exists() {
                let _ = fs::rename(&older, &newer);
            }
        }
        let v1 = path.with_extension("v1.json");
        let _ = fs::copy(&path, &v1);
    }

    let json =
        serde_json::to_string_pretty(settings).map_err(|e| format!("serialize failed: {}", e))?;
    fs::write(&path, &json).map_err(|e| format!("write failed: {}", e))?;
    Ok(())
}

pub fn get_backup_list(app: &AppHandle) -> Vec<serde_json::Value> {
    let path = settings_path(app);
    let mut backups = Vec::new();
    for i in 1..=MAX_BACKUPS {
        let bak = path.with_extension(format!("v{}.json", i));
        if bak.exists() {
            if let Ok(meta) = std::fs::metadata(&bak) {
                if let Ok(modified) = meta.modified() {
                    if let Ok(dur) = modified.duration_since(std::time::UNIX_EPOCH) {
                        let ts = chrono::DateTime::from_timestamp(dur.as_secs() as i64, 0)
                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                            .unwrap_or_default();
                        backups.push(serde_json::json!({
                            "version": i,
                            "time": ts,
                            "size": meta.len()
                        }));
                    }
                }
            }
        }
    }
    backups
}

pub fn restore_backup(app: &AppHandle, version: u32) -> Result<String, String> {
    if version < 1 || version > MAX_BACKUPS {
        return Err(format!("version must be 1-{}", MAX_BACKUPS));
    }
    let path = settings_path(app);
    let bak = path.with_extension(format!("v{}.json", version));
    if !bak.exists() {
        return Err(format!("backup v{} not found", version));
    }
    let bak_data = fs::read_to_string(&bak).map_err(|e| format!("failed to read backup: {}", e))?;
    let settings: AppSettings =
        serde_json::from_str(&bak_data).map_err(|_| "backup file corrupted".to_string())?;
    write_settings(app, &settings)?;
    Ok(format!("restored from backup v{}", version))
}
