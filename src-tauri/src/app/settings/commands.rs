use super::diagnostics::{collect_diagnostics, get_diagnostics_report, Diagnostics};
use super::io::{ensure_data_dir, get_backup_list, load_settings, restore_backup, write_settings};
use super::traits::ModuleSettings;
use super::types::AppSettings;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use tauri::{command, AppHandle, Manager};

// ========== Settings CRUD ==========

#[command]
pub fn get_settings(app: AppHandle) -> AppSettings {
    eprintln!("[Pake] get_settings called from webview");
    load_settings(&app)
}

#[command]
pub fn reset_settings(app: AppHandle) -> Result<(), String> {
    let defaults = AppSettings::default();
    write_settings(&app, &defaults)?;
    Ok(())
}

#[command]
pub fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    write_settings(&app, &settings)?;

    // Sync adblock custom rules to the engine if it's running
    if let Some(state) = app.try_state::<crate::adblock::AdblockState>() {
        let current = state.engine.custom_rules();
        if current != settings.adblock.custom_rules {
            // Clear old custom rules and add new ones
            for rule in &current {
                state.engine.remove_custom_rule(rule);
            }
            for rule in &settings.adblock.custom_rules {
                let _ = state.engine.add_custom_rule(rule);
            }
            eprintln!(
                "[Pake] adblock custom rules synced: {} rules",
                settings.adblock.custom_rules.len()
            );
        }
    }

    // Sync cache config to the engine if it's running
    crate::cache::sync_cache_config(&app, settings.cache.enabled, settings.cache.max_size_mb);

    Ok(())
}

#[command]
pub fn validate_settings(settings: AppSettings) -> Result<(), Vec<String>> {
    let mut errors: Vec<String> = Vec::new();
    errors.extend(settings.adblock.validate());
    errors.extend(settings.cache.validate());
    errors.extend(settings.clipboard.validate());
    errors.extend(settings.general.validate());
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[command]
pub fn get_module_stats(app: AppHandle) -> serde_json::Value {
    let s = load_settings(&app);
    serde_json::json!({
        "adblock": { "summary": s.adblock.summary(), "stats": s.adblock.stats() },
        "cache": { "summary": s.cache.summary(), "stats": s.cache.stats() },
        "clipboard": { "summary": s.clipboard.summary(), "stats": s.clipboard.stats() },
        "general": { "summary": s.general.summary(), "stats": s.general.stats() }
    })
}

// ========== File dialogs ==========

#[command]
pub fn pick_save_path() -> Result<String, String> {
    rfd::FileDialog::new()
        .set_title("Save exported data")
        .set_file_name("pake-data.zip")
        .add_filter("ZIP files", &["zip"])
        .save_file()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| "cancelled".into())
}

#[command]
pub fn pick_zip_file() -> Result<String, String> {
    rfd::FileDialog::new()
        .set_title("Select data file to import")
        .add_filter("ZIP files", &["zip"])
        .pick_file()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| "cancelled".into())
}

/// Returns the default download directory path for showing in UI
#[command]
pub fn get_download_dir(app: AppHandle) -> Result<String, String> {
    let dir = app
        .path()
        .download_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    Ok(dir.to_string_lossy().to_string())
}

#[command]
pub fn get_default_export_path(app: AppHandle) -> Result<String, String> {
    let default_name = format!("pake-data-{}.zip", chrono::Local::now().format("%Y%m%d"));
    let path = app
        .path()
        .download_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(&default_name);
    Ok(path.to_string_lossy().to_string())
}

// ========== Export / Import ==========

fn collect_files(
    dir: &PathBuf,
    file_list: &mut Vec<String>,
    total_size: &mut u64,
) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).map_err(|e| format!("read dir: {}", e))? {
        let entry = entry.map_err(|e| format!("entry: {}", e))?;
        let path = entry.path();
        if path.is_file() {
            if let Ok(meta) = path.metadata() {
                *total_size += meta.len();
                // Store relative path from data_dir parent
                if let (Some(parent_name), Some(file_name)) = (dir.file_name(), path.file_name()) {
                    let rel = format!(
                        "{}/{}",
                        parent_name.to_string_lossy(),
                        file_name.to_string_lossy()
                    );
                    file_list.push(rel);
                }
            }
        } else if path.is_dir() {
            collect_files(&path, file_list, total_size)?;
        }
    }
    Ok(())
}

#[command]
pub fn export_data(app: AppHandle, save_path: Option<String>) -> Result<String, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to get data dir: {}", e))?;

    // Ensure data dir exists
    fs::create_dir_all(&data_dir).map_err(|e| format!("failed to create data dir: {}", e))?;

    let default_name = format!("pake-data-{}.zip", chrono::Local::now().format("%Y%m%d"));
    let save_path = if let Some(user_path) = save_path {
        let p = PathBuf::from(&user_path);
        if p.is_dir() {
            p.join(&default_name)
        } else {
            p
        }
    } else {
        app.path()
            .download_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(&default_name)
    };

    let mut file_list: Vec<String> = Vec::new();
    let mut total_size: u64 = 0;
    if data_dir.exists() {
        for entry in
            fs::read_dir(&data_dir).map_err(|e| format!("failed to read data dir: {}", e))?
        {
            let entry = entry.map_err(|e| format!("failed to read entry: {}", e))?;
            let path = entry.path();
            if path.is_file() {
                if let Ok(meta) = path.metadata() {
                    total_size += meta.len();
                    if let Some(name) = path.file_name() {
                        file_list.push(name.to_string_lossy().to_string());
                    }
                }
            } else if path.is_dir() {
                // Walk subdirectories (e.g., clipboard/)
                collect_files(&path, &mut file_list, &mut total_size)
                    .map_err(|e| format!("failed to scan {}: {}", path.display(), e))?;
            }
        }
    }

    if file_list.is_empty() {
        return Err("no data files found to export. Save settings or use clipboard first.".into());
    }

    let zip_file =
        fs::File::create(&save_path).map_err(|e| format!("failed to create ZIP: {}", e))?;
    let mut zip_writer = zip::ZipWriter::new(zip_file);
    let zip_options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let manifest = serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "exported_at": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        "files": file_list,
        "total_size": total_size
    });
    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("failed to serialize manifest: {}", e))?;
    zip_writer
        .start_file("manifest.json", zip_options)
        .map_err(|e| format!("failed to add manifest: {}", e))?;
    zip_writer
        .write_all(manifest_json.as_bytes())
        .map_err(|e| format!("failed to write manifest: {}", e))?;

    for fname in &file_list {
        let fpath = data_dir.join(fname);
        zip_writer
            .start_file(fname.as_str(), zip_options)
            .map_err(|e| format!("failed to add {}: {}", fname, e))?;
        let mut file =
            fs::File::open(&fpath).map_err(|e| format!("failed to read {}: {}", fname, e))?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .map_err(|e| format!("failed to read {}: {}", fname, e))?;
        zip_writer
            .write_all(&buf)
            .map_err(|e| format!("failed to write {}: {}", fname, e))?;
    }

    zip_writer
        .finish()
        .map_err(|e| format!("failed to finalize ZIP: {}", e))?;

    let kb = total_size / 1024;
    Ok(format!(
        "{} file(s), {} KB → {}",
        file_list.len(),
        kb,
        save_path.to_string_lossy()
    ))
}

#[command]
pub fn preview_import(app: AppHandle, zip_path: Option<String>) -> Result<String, String> {
    let zip_path = if let Some(custom_path) = zip_path {
        let p = PathBuf::from(&custom_path);
        if !p.exists() {
            return Err(format!("file not found: {}", custom_path));
        }
        p
    } else {
        let downloads = app
            .path()
            .download_dir()
            .unwrap_or_else(|_| PathBuf::from("."));

        let mut zip_files: Vec<_> = fs::read_dir(&downloads)
            .map_err(|e| format!("failed to read downloads dir: {}", e))?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("pake-data-") && name.ends_with(".zip") {
                    let metadata = entry.metadata().ok()?;
                    let modified = metadata.modified().ok()?;
                    Some((entry.path(), modified, name))
                } else {
                    None
                }
            })
            .collect();

        if zip_files.is_empty() {
            return Err("no data file found (pake-data-*.zip) in Downloads".into());
        }

        zip_files.sort_by(|a, b| b.1.cmp(&a.1));
        zip_files.remove(0).0
    };

    let zip_name = zip_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown.zip".to_string());

    let zip_file = fs::File::open(&zip_path).map_err(|e| format!("failed to open ZIP: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(zip_file).map_err(|e| format!("failed to read ZIP archive: {}", e))?;

    let mut file_list = Vec::new();
    let mut total_size: u64 = 0;
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            if entry.is_file() {
                file_list.push(format!("  - {} ({} KB)", entry.name(), entry.size() / 1024));
                total_size += entry.size();
            }
        }
    }

    Ok(format!(
        "File: {}\nContents:\n{}\nTotal: {} file(s), {} KB\n\nExisting files will be backed up (.bak).",
        zip_name,
        file_list.join("\n"),
        file_list.len(),
        total_size / 1024
    ))
}

#[command]
pub fn import_data(app: AppHandle, zip_path: Option<String>) -> Result<String, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to get data dir: {}", e))?;

    let zip_path = if let Some(custom_path) = zip_path {
        let p = PathBuf::from(&custom_path);
        if !p.exists() {
            return Err(format!("文件不存在: {}", custom_path));
        }
        p
    } else {
        // Default: find latest in Downloads
        let downloads = app
            .path()
            .download_dir()
            .unwrap_or_else(|_| PathBuf::from("."));

        let mut zip_files: Vec<_> = fs::read_dir(&downloads)
            .map_err(|e| format!("读取下载目录失败: {}", e))?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("pake-data-") && name.ends_with(".zip") {
                    let metadata = entry.metadata().ok()?;
                    let modified = metadata.modified().ok()?;
                    Some((entry.path(), modified))
                } else {
                    None
                }
            })
            .collect();

        if zip_files.is_empty() {
            return Err("Downloads 目录中找不到 pake-data-*.zip 文件".into());
        }
        zip_files.sort_by(|a, b| b.1.cmp(&a.1));
        zip_files.remove(0).0
    };

    let temp_dir = std::env::temp_dir().join("pake-import-temp");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).map_err(|e| format!("failed to create temp dir: {}", e))?;

    let zip_file = fs::File::open(zip_path).map_err(|e| format!("failed to open ZIP: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(zip_file).map_err(|e| format!("failed to read ZIP archive: {}", e))?;

    let mut file_names = Vec::new();
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("failed to read ZIP entry: {}", e))?;
        let name = entry.name().to_string();
        if name.ends_with('/') || name.contains("..") {
            continue;
        }
        let out_path = temp_dir.join(&name);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("failed to create dir: {}", e))?;
        }
        let mut out_file =
            fs::File::create(&out_path).map_err(|e| format!("failed to create file: {}", e))?;
        std::io::copy(&mut entry, &mut out_file)
            .map_err(|e| format!("failed to extract {}: {}", name, e))?;
        file_names.push(name);
    }

    ensure_data_dir(&app);
    let mut restored = Vec::new();
    for name in &file_names {
        if name == "manifest.json" {
            continue;
        }
        let src = temp_dir.join(name);
        if src.is_file() {
            let dest = data_dir.join(name);
            if dest.exists() {
                let bak = data_dir.join(format!("{}.bak", name));
                let _ = fs::rename(&dest, &bak);
            }
            fs::copy(&src, &dest).map_err(|e| format!("failed to restore {}: {}", name, e))?;
            restored.push(name.clone());
        }
    }

    let _ = fs::remove_dir_all(&temp_dir);
    Ok(format!(
        "restored {} file(s): {}",
        restored.len(),
        restored.join(", ")
    ))
}

// ========== Backups ==========

#[command]
pub fn list_backups(app: AppHandle) -> Vec<serde_json::Value> {
    get_backup_list(&app)
}

#[command]
pub fn rollback_settings(app: AppHandle, version: u32) -> Result<String, String> {
    restore_backup(&app, version)
}

// ========== Diagnostics ==========

#[command]
pub fn get_diagnostics(app: AppHandle) -> Diagnostics {
    collect_diagnostics(&app)
}

#[command]
pub fn copy_diagnostics_report(app: AppHandle) -> Result<(), String> {
    let report = get_diagnostics_report(&app);
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("failed to open clipboard: {}", e))?;
    clipboard
        .set_text(&report)
        .map_err(|e| format!("failed to write to clipboard: {}", e))?;
    Ok(())
}
