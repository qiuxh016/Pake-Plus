use super::store::ClipboardStore;
use std::sync::Arc;
use tauri::{AppHandle, Manager, Url, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

pub const CLIPBOARD_PANEL_LABEL: &str = "clipboard-panel";
pub const CLIPBOARD_PROTOCOL: &str = "pake-clipboard";

pub fn ensure_panel_window(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    if let Some(window) = app.get_webview_window(CLIPBOARD_PANEL_LABEL) {
        return Ok(window);
    }

    let window = WebviewWindowBuilder::new(
        app,
        CLIPBOARD_PANEL_LABEL,
        WebviewUrl::CustomProtocol(Url::parse("pake-clipboard://localhost/").map_err(|error| {
            tauri::Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                error.to_string(),
            ))
        })?),
    )
    .title("剪贴板历史")
    .inner_size(420.0, 520.0)
    .min_inner_size(320.0, 360.0)
    .resizable(true)
    .decorations(true)
    .always_on_top(false)
    .visible(false)
    .skip_taskbar(false)
    .center()
    .build()?;

    Ok(window)
}

pub fn toggle_panel(app: &AppHandle) -> tauri::Result<bool> {
    let window = ensure_panel_window(app)?;
    let visible = window.is_visible().unwrap_or(false);
    if !visible {
        let _ = window.show();
        let _ = window.eval("window.pakeClipboardPanel && window.pakeClipboardPanel.refresh()");
    }
    let _ = window.set_focus();
    Ok(true)
}

pub fn show_panel(app: &AppHandle, query: Option<&str>) -> tauri::Result<()> {
    let window = ensure_panel_window(app)?;
    let query =
        serde_json::to_string(query.unwrap_or_default()).unwrap_or_else(|_| "\"\"".to_string());
    let _ = window.show();
    let _ = window.set_focus();
    let _ = window.eval(format!(
        "window.pakeClipboardPanel && window.pakeClipboardPanel.openWithQuery({query})"
    ));
    Ok(())
}

pub fn hide_panel(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(CLIPBOARD_PANEL_LABEL) {
        let _ = window.hide();
    }
}

pub fn copy_item(
    _app: &AppHandle,
    store: &Arc<ClipboardStore>,
    expected_hash: &super::monitor::ExpectedHash,
    id: i64,
) -> Result<(), String> {
    let entry = store
        .get(id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Clipboard entry not found".to_string())?;

    super::monitor::write_text_to_clipboard(&entry.content, expected_hash)?;
    Ok(())
}
