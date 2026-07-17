use super::engine::CacheEngine;
use super::CacheState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{command, AppHandle, Manager};

// ── Request / Response types ────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheFetchResponse {
    /// Whether the response came from the cache.
    pub from_cache: bool,
    /// HTTP status code (200 for cache hits, 0 for errors).
    pub status: u16,
    /// Response body as UTF-8 text (for text/* content) or empty for binary.
    pub body: String,
    /// Base64-encoded body for binary content types.
    pub body_base64: Option<String>,
    /// Content-Type header value.
    pub content_type: String,
    /// Key response headers.
    pub headers: HashMap<String, String>,
    /// Error message if something went wrong.
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CacheStatsResponse {
    pub file_count: u64,
    pub total_size: u64,
    pub total_size_mb: String,
    pub max_size: u64,
    pub max_size_mb: u64,
    pub hit_count: u64,
    pub miss_count: u64,
    pub hit_rate_1h: String,
}

// ── Cache fetch (proxy) ─────────────────────────────────────────

#[command]
pub async fn cache_fetch(
    app: AppHandle,
    url: String,
    force_cache: Option<bool>,
) -> Result<CacheFetchResponse, String> {
    let state = match app.try_state::<CacheState>() {
        Some(s) => s,
        None => {
            return Err("Cache is not enabled".to_string());
        }
    };

    if !state.enabled || force_cache == Some(false) {
        // Pass-through: fetch directly and cache the result.
        return direct_fetch_and_cache(&state.engine, &url).await;
    }

    // 1. Check cache.
    if let Some(entry) = state.engine.check_cache(&url) {
        let body = state.engine.read_body(&entry);
        if let Some(body) = body {
            let is_text = entry.content_type.starts_with("text/")
                || entry.content_type.contains("json")
                || entry.content_type.contains("javascript")
                || entry.content_type.contains("xml")
                || entry.content_type.contains("svg");
            let (body_text, body_b64) = if is_text {
                (String::from_utf8_lossy(&body).to_string(), None)
            } else {
                let b64 = base64_encode(&body);
                (String::new(), Some(b64))
            };
            return Ok(CacheFetchResponse {
                from_cache: true,
                status: 200,
                body: body_text,
                body_base64: body_b64,
                content_type: entry.content_type.clone(),
                headers: entry.response_headers.clone(),
                error: None,
            });
        }
    }

    // 2. Cache miss — fetch and cache.
    direct_fetch_and_cache(&state.engine, &url).await
}

async fn direct_fetch_and_cache(
    engine: &CacheEngine,
    url: &str,
) -> Result<CacheFetchResponse, String> {
    engine.record_miss();

    // Make the actual HTTP request.
    let client = tauri_plugin_http::reqwest::Client::builder()
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let mut response_headers = HashMap::new();
    for (key, value) in response.headers() {
        if let Ok(v) = value.to_str() {
            response_headers.insert(key.as_str().to_string(), v.to_string());
        }
    }

    // Parse Cache-Control max-age.
    let max_age = response_headers.get("cache-control").and_then(|v| {
        v.to_lowercase()
            .split(',')
            .find(|part| part.trim().starts_with("max-age"))
            .and_then(|part| {
                part.trim()
                    .strip_prefix("max-age")
                    .and_then(|s| s.trim().strip_prefix('='))
                    .and_then(|s| s.trim().parse::<u64>().ok())
            })
    });

    let body = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response body: {e}"))?;

    let body_bytes = body.to_vec();

    // Only cache successful GET responses.
    if status == 200 {
        engine.store(
            url,
            &body_bytes,
            &content_type,
            max_age,
            response_headers.clone(),
        );
    }

    let is_text = content_type.starts_with("text/")
        || content_type.contains("json")
        || content_type.contains("javascript")
        || content_type.contains("xml")
        || content_type.contains("svg");
    let (body_text, body_b64) = if is_text {
        (String::from_utf8_lossy(&body_bytes).to_string(), None)
    } else {
        let b64 = base64_encode(&body_bytes);
        (String::new(), Some(b64))
    };

    Ok(CacheFetchResponse {
        from_cache: false,
        status,
        body: body_text,
        body_base64: body_b64,
        content_type,
        headers: response_headers,
        error: None,
    })
}

// ── Cache stats ─────────────────────────────────────────────────

#[command]
pub fn cache_get_stats(app: AppHandle) -> Result<CacheStatsResponse, String> {
    let state = app
        .try_state::<CacheState>()
        .ok_or("Cache is not enabled")?;
    let stats = state.engine.stats();
    Ok(CacheStatsResponse {
        file_count: stats.file_count,
        total_size: stats.total_size,
        total_size_mb: format!("{:.1}", stats.total_size as f64 / (1024.0 * 1024.0)),
        max_size: stats.max_size,
        max_size_mb: stats.max_size / (1024 * 1024),
        hit_count: stats.hit_count,
        miss_count: stats.miss_count,
        hit_rate_1h: format!("{:.0}%", stats.hit_rate_1h * 100.0),
    })
}

// ── Cache clear ─────────────────────────────────────────────────

#[command]
pub fn cache_clear(app: AppHandle) -> Result<(), String> {
    let state = app
        .try_state::<CacheState>()
        .ok_or("Cache is not enabled")?;
    state.engine.clear_all();
    Ok(())
}

// ── Cache stats JSON (for settings panel) ───────────────────────

#[command]
pub fn cache_stats_json(app: AppHandle) -> Result<serde_json::Value, String> {
    let state = app
        .try_state::<CacheState>()
        .ok_or("Cache is not enabled")?;
    Ok(state.engine.stats_json())
}

// ── Helpers ─────────────────────────────────────────────────────

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}
