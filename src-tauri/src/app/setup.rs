#[cfg(feature = "clipboard")]
use crate::app::clipboard::panel::toggle_panel;
use crate::app::settings::open_settings_panel;
use crate::app::window::open_additional_window_safe;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use tauri_plugin_window_state::{AppHandleExt, StateFlags};

pub fn update_tray_status(app: &AppHandle) {
    let settings = crate::app::settings::get_settings_inner(app);
    let text = format!(
        "Adblock: {}  Cache: {}  Clipboard: {}",
        if settings.adblock.enabled {
            "ON"
        } else {
            "OFF"
        },
        if settings.cache.enabled { "ON" } else { "OFF" },
        if settings.clipboard.enabled {
            "ON"
        } else {
            "OFF"
        },
    );
    // Rebuild tray to reflect status
    if let Some(tray) = app.tray_by_id("pake-tray") {
        let _ = tray.set_tooltip(Some(&text));
    }
}

#[cfg(not(feature = "clipboard"))]
fn toggle_panel(_app: &AppHandle) -> tauri::Result<bool> {
    Ok(false)
}

pub fn set_system_tray(
    app: &AppHandle,
    show_system_tray: bool,
    tray_icon_path: &str,
    _init_fullscreen: bool,
    allow_multi_window: bool,
    clipboard_available: bool,
    clipboard_listening: bool,
) -> tauri::Result<()> {
    if !show_system_tray {
        app.remove_tray_by_id("pake-tray");
        return Ok(());
    }

    let settings = crate::app::settings::get_settings_inner(app);
    let status_text_str = format!(
        "Adblock: {}  Cache: {}  Clipboard: {}",
        if settings.adblock.enabled {
            "ON"
        } else {
            "OFF"
        },
        if settings.cache.enabled { "ON" } else { "OFF" },
        if settings.clipboard.enabled {
            "ON"
        } else {
            "OFF"
        },
    );
    let status_text = MenuItemBuilder::with_id("status", &status_text_str).build(app)?;
    let copy_diag = MenuItemBuilder::with_id("copy_diagnostics", "Copy Diagnostics").build(app)?;
    let sep = tauri::menu::PredefinedMenuItem::separator(app)?;
    let new_window = MenuItemBuilder::with_id("new_window", "New Window").build(app)?;
    let settings = MenuItemBuilder::with_id("settings", "Settings").build(app)?;
    let clipboard_history =
        MenuItemBuilder::with_id("clipboard_history", "Clipboard History").build(app)?;
    let hide_app = MenuItemBuilder::with_id("hide_app", "Hide").build(app)?;
    let show_app = MenuItemBuilder::with_id("show_app", "Show").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

    let mut items: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> =
        vec![&status_text, &copy_diag, &sep];
    if allow_multi_window {
        items.push(&new_window);
    }
    items.push(&settings);
    if clipboard_available {
        items.push(&clipboard_history);
    }
    items.push(&hide_app);
    items.push(&show_app);
    items.push(&quit);

    let menu = MenuBuilder::new(app).items(&items).build()?;

    app.app_handle().remove_tray_by_id("pake-tray");

    let mut tray_builder = TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "status" => {
                open_settings_panel(app);
            }
            "copy_diagnostics" => {
                if let Some(window) = app.get_webview_window("pake") {
                    let _ = window.eval("window.__pakeOpenSettings()");
                }
                std::thread::spawn({
                    let app_h = app.clone();
                    move || {
                        let report = crate::app::settings::get_diagnostics_report(&app_h);
                        if let Ok(mut c) = arboard::Clipboard::new() {
                            let _ = c.set_text(&report);
                        }
                    }
                });
            }
            "new_window" => {
                open_additional_window_safe(app);
            }
            "settings" => {
                open_settings_panel(app);
            }
            "clipboard_history" => {
                let _ = toggle_panel(app);
            }
            "hide_app" => {
                if let Some(window) = app.get_webview_window("pake") {
                    let _ = window.minimize();
                }
            }
            "show_app" => {
                if let Some(window) = app.get_webview_window("pake") {
                    let _ = window.show();
                    #[cfg(target_os = "linux")]
                    if _init_fullscreen && !window.is_fullscreen().unwrap_or(false) {
                        let _ = window.set_fullscreen(true);
                        let _ = window.set_focus();
                    }
                }
            }
            "quit" => {
                let flags = if _init_fullscreen {
                    StateFlags::all()
                } else {
                    StateFlags::all() & !StateFlags::FULLSCREEN
                };
                let _ = app.save_window_state(flags);
                app.exit(0);
            }
            _ => (),
        })
        .on_tray_icon_event(move |tray, event| {
            if let TrayIconEvent::Click { button, .. } = event {
                if button == tauri::tray::MouseButton::Left {
                    if let Some(window) = tray.app_handle().get_webview_window("pake") {
                        let is_visible = window.is_visible().unwrap_or(false);
                        if is_visible {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                            #[cfg(target_os = "linux")]
                            if _init_fullscreen && !window.is_fullscreen().unwrap_or(false) {
                                let _ = window.set_fullscreen(true);
                            }
                        }
                    }
                }
            }
        });

    let resolved_icon = if tray_icon_path.is_empty() {
        app.default_window_icon().cloned()
    } else {
        tauri::image::Image::from_path(tray_icon_path)
            .ok()
            .or_else(|| app.default_window_icon().cloned())
    };

    if let Some(icon) = resolved_icon.map(|icon| {
        if clipboard_listening {
            add_listening_indicator(icon)
        } else {
            icon
        }
    }) {
        tray_builder = tray_builder.icon(icon);
    } else {
        eprintln!("[Pake] No tray icon available; tray will build without an icon.");
    }

    if clipboard_listening {
        tray_builder = tray_builder.tooltip("● Clipboard listening");
    }

    let tray = tray_builder.build(app)?;

    tray.set_icon_as_template(false)?;
    Ok(())
}

fn add_listening_indicator(icon: Image<'_>) -> Image<'static> {
    let width = icon.width();
    let height = icon.height();
    let mut rgba = icon.rgba().to_vec();
    let radius = (width.min(height) / 7).max(2);
    let center_x = width.saturating_sub(radius + 1);
    let center_y = height.saturating_sub(radius + 1);
    let radius_squared = (radius * radius) as i64;

    for y in center_y.saturating_sub(radius)..(center_y + radius).min(height) {
        for x in center_x.saturating_sub(radius)..(center_x + radius).min(width) {
            let dx = x as i64 - center_x as i64;
            let dy = y as i64 - center_y as i64;
            if dx * dx + dy * dy <= radius_squared {
                let index = ((y * width + x) * 4) as usize;
                if index + 3 < rgba.len() {
                    rgba[index..index + 4].copy_from_slice(&[48, 209, 88, 255]);
                }
            }
        }
    }
    Image::new_owned(rgba, width, height)
}

#[cfg(feature = "clipboard")]
pub fn update_clipboard_tray_state(app: &AppHandle, listening: bool) {
    if let Some(tray) = app.tray_by_id("pake-tray") {
        if listening {
            let _ = tray.set_tooltip(Some("● Clipboard listening"));
        } else {
            let _ = tray.set_tooltip(None::<&str>);
        }
    }
}

pub fn set_global_shortcut(
    app: &AppHandle,
    shortcut: String,
    _init_fullscreen: bool,
    clipboard_enabled: bool,
) -> tauri::Result<()> {
    if shortcut.is_empty() && !clipboard_enabled {
        return Ok(());
    }

    let mut activation_hotkey = if shortcut.is_empty() {
        None
    } else {
        match Shortcut::from_str(&shortcut) {
            Ok(value) => Some(value),
            Err(error) => {
                eprintln!("[Pake] Invalid activation shortcut '{shortcut}': {error}");
                None
            }
        }
    };

    let clipboard_hotkey = if clipboard_enabled {
        let shortcut_text = if cfg!(target_os = "macos") {
            "Cmd+Shift+V"
        } else {
            "Ctrl+Shift+V"
        };
        match Shortcut::from_str(shortcut_text) {
            Ok(value) => Some(value),
            Err(error) => {
                eprintln!("[Pake] Invalid clipboard shortcut '{shortcut_text}': {error}");
                None
            }
        }
    } else {
        None
    };

    if activation_hotkey.is_some() && activation_hotkey == clipboard_hotkey {
        eprintln!(
            "[Pake] Activation shortcut conflicts with clipboard history; clipboard history takes priority."
        );
        activation_hotkey = None;
    }

    if activation_hotkey.is_none() && clipboard_hotkey.is_none() {
        return Ok(());
    }

    let last_triggered = Arc::new(Mutex::new(Instant::now()));

    if let Err(error) = app.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler({
                let last_triggered = Arc::clone(&last_triggered);
                move |app, event, _registered| {
                    let Ok(mut last_triggered) = last_triggered.lock() else {
                        return;
                    };
                    if Instant::now().duration_since(*last_triggered) < Duration::from_millis(300) {
                        return;
                    }
                    *last_triggered = Instant::now();

                    if activation_hotkey
                        .as_ref()
                        .is_some_and(|hotkey| hotkey == event)
                    {
                        if let Some(window) = app.get_webview_window("pake") {
                            let is_visible = window.is_visible().unwrap_or(false);
                            if is_visible {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                                #[cfg(target_os = "linux")]
                                if _init_fullscreen && !window.is_fullscreen().unwrap_or(false) {
                                    let _ = window.set_fullscreen(true);
                                }
                            }
                        }
                    }

                    if clipboard_hotkey
                        .as_ref()
                        .is_some_and(|hotkey| hotkey == event)
                    {
                        let _ = toggle_panel(app);
                    }
                }
            })
            .build(),
    ) {
        eprintln!(
            "[Pake] Failed to register global shortcut plugin: {error}; continuing without it."
        );
        return Ok(());
    }

    if let Some(hotkey) = activation_hotkey {
        if let Err(error) = app.global_shortcut().register(hotkey) {
            eprintln!("[Pake] Failed to bind activation shortcut '{shortcut}': {error}");
        }
    }

    if let Some(hotkey) = clipboard_hotkey {
        if let Err(error) = app.global_shortcut().register(hotkey) {
            eprintln!("[Pake] Failed to bind clipboard shortcut: {error}");
        }
    }

    Ok(())
}
