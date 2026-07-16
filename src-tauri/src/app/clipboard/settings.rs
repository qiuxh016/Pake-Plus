use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_MAX_ITEMS: u32 = 2_000;
pub const MIN_MAX_ITEMS: u32 = 500;
pub const MAX_MAX_ITEMS: u32 = 5_000;
pub const DEFAULT_RETENTION_DAYS: u32 = 30;
pub const DEFAULT_MIN_LENGTH: usize = 2;
pub const DEFAULT_MAX_LENGTH: usize = 10_000;
pub const MAX_IGNORED_APPS: usize = 100;
pub const MAX_APP_NAME_LENGTH: usize = 128;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ClipboardSettings {
    pub enabled: bool,
    pub max_items: u32,
    pub retention_days: u32,
    pub ignore_short_text: bool,
    pub min_length: usize,
    pub max_length: usize,
    pub ignore_password_like: bool,
    pub ignore_credit_card_like: bool,
    pub ignored_apps: Vec<String>,
}

impl Default for ClipboardSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            max_items: DEFAULT_MAX_ITEMS,
            retention_days: DEFAULT_RETENTION_DAYS,
            ignore_short_text: true,
            min_length: DEFAULT_MIN_LENGTH,
            max_length: DEFAULT_MAX_LENGTH,
            ignore_password_like: true,
            ignore_credit_card_like: true,
            ignored_apps: Vec::new(),
        }
    }
}

impl ClipboardSettings {
    pub fn from_build_config(clipboard: bool, clipboard_max: u32) -> Self {
        Self {
            enabled: clipboard,
            max_items: normalize_max_items(clipboard_max),
            ..Self::default()
        }
    }

    pub fn load(path: &Path, clipboard: bool, clipboard_max: u32) -> Self {
        let defaults = Self::from_build_config(clipboard, clipboard_max);
        let Ok(content) = fs::read_to_string(path) else {
            return defaults;
        };

        match serde_json::from_str::<ClipboardSettings>(&content) {
            Ok(mut settings) => {
                settings.enabled = clipboard;
                settings.max_items = normalize_max_items(clipboard_max);
                settings.normalized(clipboard)
            }
            Err(error) => {
                eprintln!(
                    "[Pake] Invalid clipboard settings at {}: {error}; restoring defaults",
                    path.display()
                );
                defaults
            }
        }
    }

    pub fn normalized(mut self, build_enabled: bool) -> Self {
        self.enabled &= build_enabled;
        self.max_items = normalize_max_items(self.max_items);
        self.retention_days = DEFAULT_RETENTION_DAYS;
        self.min_length = DEFAULT_MIN_LENGTH;
        self.max_length = DEFAULT_MAX_LENGTH;
        self.ignored_apps = normalize_ignored_apps(self.ignored_apps);
        self
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        let temporary_path = path.with_extension("json.tmp");
        fs::write(&temporary_path, content)?;

        match fs::rename(&temporary_path, path) {
            Ok(()) => Ok(()),
            Err(_error) if path.exists() => {
                fs::remove_file(path)?;
                fs::rename(temporary_path, path)
            }
            Err(error) => Err(error),
        }
    }
}

pub fn normalize_max_items(value: u32) -> u32 {
    value.clamp(MIN_MAX_ITEMS, MAX_MAX_ITEMS)
}

fn normalize_ignored_apps(values: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for value in values {
        let value: String = value.trim().chars().take(MAX_APP_NAME_LENGTH).collect();
        if value.is_empty()
            || normalized
                .iter()
                .any(|existing: &String| existing.eq_ignore_ascii_case(&value))
        {
            continue;
        }
        normalized.push(value);
        if normalized.len() == MAX_IGNORED_APPS {
            break;
        }
    }
    normalized
}

pub fn settings_path(data_dir: &Path) -> PathBuf {
    data_dir.join("clipboard-settings.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_limits_and_build_gate() {
        let settings = ClipboardSettings {
            enabled: true,
            max_items: 50,
            retention_days: 0,
            ignore_short_text: false,
            min_length: 1,
            max_length: 50_000,
            ignore_password_like: false,
            ignore_credit_card_like: false,
            ignored_apps: vec!["  Notepad.exe  ".to_string(), "notepad.exe".to_string()],
        }
        .normalized(false);

        assert!(!settings.enabled);
        assert_eq!(settings.max_items, MIN_MAX_ITEMS);
        assert_eq!(settings.retention_days, DEFAULT_RETENTION_DAYS);
        assert!(!settings.ignore_short_text);
        assert_eq!(settings.min_length, DEFAULT_MIN_LENGTH);
        assert_eq!(settings.max_length, DEFAULT_MAX_LENGTH);
        assert!(!settings.ignore_password_like);
        assert!(!settings.ignore_credit_card_like);
        assert_eq!(settings.ignored_apps, vec!["Notepad.exe"]);
    }

    #[test]
    fn build_defaults_use_requested_maximum() {
        let settings = ClipboardSettings::from_build_config(true, 4_000);
        assert!(settings.enabled);
        assert_eq!(settings.max_items, 4_000);
        assert_eq!(settings.retention_days, DEFAULT_RETENTION_DAYS);
    }

    #[test]
    fn corrupted_settings_fall_back_to_build_defaults() {
        let path = std::env::temp_dir().join(format!(
            "pake-clipboard-settings-{}-{}.json",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        std::fs::write(&path, "{not valid json").unwrap();

        let settings = ClipboardSettings::load(&path, true, 3_000);
        assert_eq!(settings, ClipboardSettings::from_build_config(true, 3_000));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn build_flags_override_persisted_enablement_and_capacity() {
        let path = std::env::temp_dir().join(format!(
            "pake-clipboard-settings-{}-{}.json",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let mut persisted = ClipboardSettings::from_build_config(false, 2_000);
        persisted.ignore_password_like = false;
        persisted.save(&path).unwrap();

        let settings = ClipboardSettings::load(&path, true, 5_000);
        assert!(settings.enabled);
        assert_eq!(settings.max_items, 5_000);
        assert!(!settings.ignore_password_like);
        let _ = std::fs::remove_file(path);
    }
}
