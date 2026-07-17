#[cfg_attr(mobile, tauri::mobile_entry_point)]
mod adblock;
mod app;
mod cache;
mod util;

#[cfg(feature = "clipboard")]
use tauri::Listener;
use tauri::Manager;

use tauri_plugin_window_state::Builder as WindowStatePlugin;
use tauri_plugin_window_state::StateFlags;

#[cfg(target_os = "macos")]
use std::time::Duration;

const WINDOW_SHOW_DELAY: u64 = 50;
#[cfg(target_os = "linux")]
const PAKE_LINUX_WEBKIT_SAFE_MODE: &str = "PAKE_LINUX_WEBKIT_SAFE_MODE";
#[cfg(target_os = "linux")]
const WEBKIT_DISABLE_DMABUF_RENDERER: &str = "WEBKIT_DISABLE_DMABUF_RENDERER";
#[cfg(target_os = "linux")]
const WEBKIT_DISABLE_COMPOSITING_MODE: &str = "WEBKIT_DISABLE_COMPOSITING_MODE";
#[cfg(target_os = "linux")]
const GDK_BACKEND: &str = "GDK_BACKEND";

use crate::adblock::AdblockState;
use crate::cache::commands::{cache_clear, cache_fetch, cache_get_stats, cache_stats_json};
use crate::cache::CacheState;
#[cfg(feature = "clipboard")]
use app::clipboard::{
    clipboard_clear_all, clipboard_copy_item, clipboard_delete_item, clipboard_get_settings,
    clipboard_hide_panel, clipboard_list, clipboard_search, clipboard_show_panel, clipboard_stats,
    clipboard_update_settings, init_clipboard_state,
};
use app::{
    invoke::{
        adblock_add_rule, adblock_get_stats, adblock_remove_rule, adblock_report_blocked,
        clear_dock_badge, download_file, increment_dock_badge, send_notification, set_dock_badge,
        set_dock_badge_label, set_zoom, update_theme_mode,
    },
    settings::{
        copy_diagnostics_report, export_data, get_diagnostics, get_download_dir, get_module_stats,
        get_settings, import_data, list_backups, pick_save_path, pick_zip_file, preview_import,
        reset_settings, rollback_settings, save_settings, validate_settings,
    },
    setup::{set_global_shortcut, set_system_tray},
    window::{open_additional_window_safe, set_window, MultiWindowState},
};
use util::{get_data_dir, get_pake_config};

#[cfg(any(target_os = "linux", test))]
fn is_disabled_env_value(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "0" | "false" | "off" | "no" | "native" | "disabled"
    )
}

#[cfg(any(target_os = "linux", test))]
fn is_non_empty_env_value(value: Option<&str>) -> bool {
    value.map(|value| !value.trim().is_empty()).unwrap_or(false)
}

#[cfg(any(target_os = "linux", test))]
fn contains_niri(value: &str) -> bool {
    value
        .split([':', ';', ',', ' '])
        .any(|part| part.eq_ignore_ascii_case("niri"))
}

#[cfg(any(target_os = "linux", test))]
fn should_enable_linux_webkit_safe_mode_from_values(
    safe_mode: Option<&str>,
    niri_socket: Option<&str>,
    desktop_values: &[Option<&str>],
) -> bool {
    if let Some(value) = safe_mode.filter(|value| !value.trim().is_empty()) {
        return !is_disabled_env_value(value);
    }

    let is_niri_session = is_non_empty_env_value(niri_socket)
        || desktop_values
            .iter()
            .flatten()
            .any(|value| contains_niri(value));

    !is_niri_session
}

#[cfg(any(target_os = "linux", test))]
fn should_force_wayland_gdk_backend(
    gdk_backend: Option<&str>,
    wayland_display: Option<&str>,
    display: Option<&str>,
) -> bool {
    // Respect an explicit user choice.
    if is_non_empty_env_value(gdk_backend) {
        return false;
    }

    // On pure Wayland compositors without XWayland (e.g. Niri), $DISPLAY is unset
    // and GTK defaults to the X11 backend, which aborts with "Failed to initialize
    // GTK". Wayland is then the only viable backend, so forcing it is safe.
    is_non_empty_env_value(wayland_display) && !is_non_empty_env_value(display)
}

#[cfg(target_os = "linux")]
fn apply_linux_gdk_backend() {
    if should_force_wayland_gdk_backend(
        std::env::var(GDK_BACKEND).ok().as_deref(),
        std::env::var("WAYLAND_DISPLAY").ok().as_deref(),
        std::env::var("DISPLAY").ok().as_deref(),
    ) {
        std::env::set_var(GDK_BACKEND, "wayland");
    }
}

#[cfg(target_os = "linux")]
fn apply_linux_webkit_runtime_flags() {
    let safe_mode = std::env::var(PAKE_LINUX_WEBKIT_SAFE_MODE).ok();
    if safe_mode.as_deref().is_some_and(is_disabled_env_value) {
        std::env::remove_var(WEBKIT_DISABLE_DMABUF_RENDERER);
        std::env::remove_var(WEBKIT_DISABLE_COMPOSITING_MODE);
        return;
    }

    let desktop_values = [
        std::env::var("XDG_CURRENT_DESKTOP").ok(),
        std::env::var("XDG_SESSION_DESKTOP").ok(),
        std::env::var("DESKTOP_SESSION").ok(),
    ];
    let desktop_refs = desktop_values
        .iter()
        .map(|value| value.as_deref())
        .collect::<Vec<_>>();

    if !should_enable_linux_webkit_safe_mode_from_values(
        safe_mode.as_deref(),
        std::env::var("NIRI_SOCKET").ok().as_deref(),
        &desktop_refs,
    ) {
        return;
    }

    if std::env::var(WEBKIT_DISABLE_DMABUF_RENDERER).is_err() {
        std::env::set_var(WEBKIT_DISABLE_DMABUF_RENDERER, "1");
    }
    if std::env::var(WEBKIT_DISABLE_COMPOSITING_MODE).is_err() {
        std::env::set_var(WEBKIT_DISABLE_COMPOSITING_MODE, "1");
    }
}

pub fn run_app() {
    #[cfg(target_os = "linux")]
    {
        apply_linux_gdk_backend();
        apply_linux_webkit_runtime_flags();
    }

    let (pake_config, tauri_config) = get_pake_config();
    let tauri_app = tauri::Builder::default();

    let show_system_tray = pake_config.show_system_tray();
    let hide_on_close = pake_config.windows[0].hide_on_close;
    let activation_shortcut = pake_config.windows[0].activation_shortcut.clone();
    let init_fullscreen = pake_config.windows[0].fullscreen;
    let start_to_tray = pake_config.windows[0].start_to_tray && show_system_tray; // Only valid when tray is enabled
    let multi_instance = pake_config.multi_instance;
    let multi_window = pake_config.multi_window;
    #[cfg(feature = "clipboard")]
    let clipboard_enabled = pake_config.clipboard;
    #[cfg(feature = "clipboard")]
    let clipboard_max = pake_config.clipboard_max;
    #[cfg(feature = "clipboard")]
    eprintln!(
        "[Pake] clipboard_enabled={} clipboard_max={}",
        clipboard_enabled, clipboard_max
    );
    let _enable_find = pake_config.windows[0].enable_find;
    let block_ads = pake_config.block_ads;
    let adblock_rules = pake_config.adblock_rules.clone();
    let cache_enabled = pake_config.cache;
    let cache_size = pake_config.cache_size;
    let package_name = tauri_config
        .product_name
        .clone()
        .unwrap_or_else(|| "pake".to_string());
    let window_state_plugin = WindowStatePlugin::default()
        .with_state_flags(if init_fullscreen {
            StateFlags::FULLSCREEN
        } else {
            // Prevent flickering on the first open.
            // Exclude FULLSCREEN so a prior --fullscreen build's persisted state
            // doesn't force fullscreen on a rebuild without --fullscreen.
            StateFlags::all() & !StateFlags::VISIBLE & !StateFlags::FULLSCREEN
        })
        .build();

    #[allow(deprecated)]
    let mut app_builder = tauri_app
        .plugin(window_state_plugin)
        .plugin(tauri_plugin_oauth::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init()); // Add this

    // Only add single instance plugin if multiple instances are not allowed
    if !multi_instance {
        app_builder = app_builder.plugin(tauri_plugin_single_instance::init(
            move |app, _args, _cwd| {
                if multi_window {
                    open_additional_window_safe(app);
                } else if let Some(window) = app.get_webview_window("pake") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            },
        ));
    }

    #[cfg(feature = "clipboard")]
    {
        app_builder = app_builder
            .register_uri_scheme_protocol(
                app::clipboard::panel::CLIPBOARD_PROTOCOL,
                |context, _request| {
                    let allowed =
                        context.webview_label() == app::clipboard::panel::CLIPBOARD_PANEL_LABEL;
                    let (status, body) = if allowed {
                        (
                            200,
                            include_bytes!("../assets/clipboard-panel.html").as_slice(),
                        )
                    } else {
                        (403, b"Forbidden".as_slice())
                    };
                    tauri::http::Response::builder()
                        .status(status)
                        .header("Content-Type", "text/html; charset=utf-8")
                        .header("Cache-Control", "no-store")
                        .body(body.to_vec())
                        .expect("valid clipboard protocol response")
                },
            )
            .invoke_handler(tauri::generate_handler![
                download_file,
                send_notification,
                increment_dock_badge,
                set_dock_badge,
                set_dock_badge_label,
                clear_dock_badge,
                update_theme_mode,
                set_zoom,
                get_settings,
                save_settings,
                reset_settings,
                validate_settings,
                get_module_stats,
                export_data,
                import_data,
                preview_import,
                rollback_settings,
                list_backups,
                get_diagnostics,
                copy_diagnostics_report,
                clipboard_list,
                clipboard_search,
                clipboard_copy_item,
                clipboard_delete_item,
                clipboard_clear_all,
                clipboard_stats,
                clipboard_hide_panel,
                clipboard_show_panel,
                clipboard_get_settings,
                clipboard_update_settings,
                adblock_report_blocked,
                adblock_get_stats,
                adblock_add_rule,
                adblock_remove_rule,
                pick_save_path,
                pick_zip_file,
                cache_fetch,
                cache_get_stats,
                cache_clear,
                cache_stats_json,
            ]);
    }

    // Settings panel protocol — always available regardless of clipboard feature
    app_builder =
        app_builder.register_uri_scheme_protocol("pake-settings", |_context, _request| {
            tauri::http::Response::builder()
                .status(200)
                .header("Content-Type", "text/html; charset=utf-8")
                .header("Cache-Control", "no-store")
                .body(include_bytes!("../assets/settings.html").to_vec())
                .expect("valid settings protocol response")
        });

    #[cfg(not(feature = "clipboard"))]
    {
        app_builder = app_builder.invoke_handler(tauri::generate_handler![
            download_file,
            send_notification,
            increment_dock_badge,
            set_dock_badge,
            set_dock_badge_label,
            clear_dock_badge,
            update_theme_mode,
            set_zoom,
            get_settings,
            save_settings,
            reset_settings,
            validate_settings,
            get_module_stats,
            export_data,
            import_data,
            preview_import,
            rollback_settings,
            list_backups,
            get_diagnostics,
            copy_diagnostics_report,
            adblock_report_blocked,
            adblock_get_stats,
            adblock_add_rule,
            adblock_remove_rule,
            pick_save_path,
            pick_zip_file,
            cache_fetch,
            cache_get_stats,
            cache_clear,
            cache_stats_json,
        ]);
    }

    app_builder
        .setup(move |app| {
            app.manage(MultiWindowState::new(
                pake_config.clone(),
                tauri_config.clone(),
            ));

            // --- Menu Construction Start ---
            #[cfg(target_os = "macos")]
            {
                app::menu::set_app_menu(app.app_handle(), multi_window, _enable_find)?;

                // Event Handling for Custom Menu Item
                app.on_menu_event(move |app_handle, event| {
                    app::menu::handle_menu_click(app_handle, event.id().as_ref());
                });
            }
            // --- Menu Construction End ---

            // Init adblock state before set_window so it's available during webview creation
            if block_ads {
                let adblock_state = AdblockState::new(true, &adblock_rules);
                app.manage(adblock_state);
            }

            // Init cache state before set_window so cache.js can be injected
            if cache_enabled {
                let cache_dir = get_data_dir(app.app_handle(), package_name.clone())
                    .map(|d| d.join("cache"))
                    .map_err(|e| tauri::Error::Io(e))?;
                let cache_state = CacheState::new(cache_dir, true, cache_size);
                app.manage(cache_state);
            }

            let window = set_window(app.app_handle(), &pake_config, &tauri_config)?;

            #[cfg(feature = "clipboard")]
            let clipboard_listening = if clipboard_enabled {
                let clipboard_state = init_clipboard_state(
                    app.app_handle(),
                    clipboard_enabled,
                    clipboard_max,
                    package_name.clone(),
                )
                .map_err(|error| tauri::Error::Io(std::io::Error::other(error)))?;
                let clipboard_listening = clipboard_state.is_enabled();
                app.manage(clipboard_state);
                clipboard_listening
            } else {
                false
            };
            #[cfg(not(feature = "clipboard"))]
            let clipboard_listening = false;

            set_system_tray(
                app.app_handle(),
                show_system_tray,
                &pake_config.system_tray_path,
                init_fullscreen,
                multi_window,
                clipboard_listening,
                clipboard_listening,
            )?;
            set_global_shortcut(
                app.app_handle(),
                activation_shortcut,
                init_fullscreen,
                clipboard_listening,
            )?;
            app::setup::update_tray_status(app.app_handle());
            let health = app::settings::run_health_check(app.app_handle());
            for msg in &health {
                eprintln!("[Pake] Health: {}", msg);
            }

            // Listen for frontend event to open clipboard panel
            #[cfg(feature = "clipboard")]
            {
                let handle = app.app_handle().clone();
                app.app_handle()
                    .listen("open-clipboard-panel", move |_event| {
                        let _ = app::clipboard::panel::toggle_panel(&handle);
                    });
            }

            // Show window after state restoration to prevent position flashing
            // Unless start_to_tray is enabled, then keep it hidden
            if !start_to_tray {
                let window_clone = window.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(WINDOW_SHOW_DELAY)).await;
                    let _ = window_clone.show();

                    // Fixed: Linux fullscreen issue with virtual keyboard
                    #[cfg(target_os = "linux")]
                    {
                        if init_fullscreen {
                            let _ = window_clone.set_fullscreen(true);
                            // Ensure webview maintains focus for input after fullscreen
                            let _ = window_clone.set_focus();
                        } else {
                            // Fix: Ubuntu 24.04/GNOME window buttons non-functional until resize (#1122)
                            // The window manager needs time to process the MapWindow event before
                            // accepting focus requests. Without this, decorations remain non-interactive.
                            tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
                            let _ = window_clone.set_focus();
                        }
                    }
                });
            }

            Ok(())
        })
        .on_window_event(move |_window, _event| {
            #[cfg(feature = "clipboard")]
            if _window.label() == app::clipboard::panel::CLIPBOARD_PANEL_LABEL {
                if let tauri::WindowEvent::CloseRequested { api, .. } = _event {
                    let _ = _window.hide();
                    api.prevent_close();
                }
            }
            if let tauri::WindowEvent::CloseRequested { api, .. } = _event {
                if hide_on_close && _window.label() == "pake" {
                    // Hide window when hide_on_close is enabled (regardless of tray status)
                    let window = _window.clone();
                    tauri::async_runtime::spawn(async move {
                        #[cfg(target_os = "macos")]
                        {
                            if window.is_fullscreen().unwrap_or(false) {
                                let _ = window.set_fullscreen(false);
                                tokio::time::sleep(Duration::from_millis(900)).await;
                            }
                        }
                        #[cfg(target_os = "linux")]
                        {
                            if window.is_fullscreen().unwrap_or(false) {
                                let _ = window.set_fullscreen(false);
                                // Restore focus after exiting fullscreen to fix input issues
                                let _ = window.set_focus();
                            }
                        }
                        // On macOS, directly hide without minimize to avoid duplicate Dock icons
                        #[cfg(not(target_os = "macos"))]
                        let _ = window.minimize();
                        let _ = window.hide();
                    });
                    api.prevent_close();
                }
                // If hide_on_close is false, allow normal close behavior
                // This lets tauri-plugin-window-state save the window position and size
            }
        })
        .build(tauri::generate_context!())
        .unwrap_or_else(|error| {
            eprintln!("[Pake] Fatal error while building Tauri application: {error}");
            std::process::exit(1);
        })
        .run(|_app, _event| {
            // Handle macOS dock icon click to reopen hidden window
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen {
                has_visible_windows,
                ..
            } = _event
            {
                if !has_visible_windows {
                    if let Some(window) = _app.get_webview_window("pake") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        });
}

pub fn run() {
    run_app()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_webkit_safe_mode_stays_on_by_default() {
        assert!(should_enable_linux_webkit_safe_mode_from_values(
            None,
            None,
            &[None, None, None]
        ));
    }

    #[test]
    fn linux_webkit_safe_mode_is_disabled_for_niri_socket() {
        assert!(!should_enable_linux_webkit_safe_mode_from_values(
            None,
            Some("/run/user/501/niri.sock"),
            &[None, None, None]
        ));
    }

    #[test]
    fn linux_webkit_safe_mode_is_disabled_for_niri_desktop() {
        assert!(!should_enable_linux_webkit_safe_mode_from_values(
            None,
            None,
            &[Some("niri"), None, None]
        ));
    }

    #[test]
    fn linux_webkit_safe_mode_can_be_forced_on_for_niri() {
        assert!(should_enable_linux_webkit_safe_mode_from_values(
            Some("1"),
            Some("/run/user/501/niri.sock"),
            &[Some("niri"), None, None]
        ));
    }

    #[test]
    fn linux_webkit_safe_mode_can_be_disabled_explicitly() {
        for value in ["0", "false", "off", "no", "native", "disabled"] {
            assert!(
                !should_enable_linux_webkit_safe_mode_from_values(
                    Some(value),
                    None,
                    &[None, None, None]
                ),
                "expected {value} to disable safe mode"
            );
        }
    }

    #[test]
    fn forces_wayland_backend_on_pure_wayland() {
        assert!(should_force_wayland_gdk_backend(
            None,
            Some("wayland-0"),
            None
        ));
    }

    #[test]
    fn forces_wayland_backend_when_display_is_blank() {
        assert!(should_force_wayland_gdk_backend(
            None,
            Some("wayland-0"),
            Some("   ")
        ));
    }

    #[test]
    fn keeps_default_backend_when_x11_display_present() {
        assert!(!should_force_wayland_gdk_backend(
            None,
            Some("wayland-0"),
            Some(":0")
        ));
    }

    #[test]
    fn keeps_default_backend_without_wayland_display() {
        assert!(!should_force_wayland_gdk_backend(None, None, None));
    }

    #[test]
    fn respects_explicit_gdk_backend_override() {
        assert!(!should_force_wayland_gdk_backend(
            Some("x11"),
            Some("wayland-0"),
            None
        ));
    }
}
