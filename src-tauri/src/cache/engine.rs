use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

// ── Cache limits ────────────────────────────────────────────────

/// Maximum size for a single cached file (10 MB).
pub const MAX_SINGLE_FILE: u64 = 10 * 1024 * 1024;

/// Default cache TTL when the server doesn't provide one (24 hours in seconds).
pub const DEFAULT_TTL_SECS: u64 = 86_400;

/// Fraction of total cache to evict when over limit (5% = ~10MB for 200MB).
const EVICT_FRACTION: f64 = 0.05;

/// Minimum eviction size regardless of fraction.
const MIN_EVICT_SIZE: u64 = 10 * 1024 * 1024; // 10 MB

// ── Cache entry ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Original URL that was cached.
    pub url: String,
    /// SHA-256 hex digest of the URL — used as the on-disk filename.
    pub hash: String,
    /// Content-Type header from the original response.
    pub content_type: String,
    /// Unix timestamp (seconds) when this entry was first cached.
    pub cached_at: u64,
    /// Unix timestamp (seconds) of the most recent cache hit.
    pub last_accessed: u64,
    /// File size in bytes.
    pub size: u64,
    /// When the entry expires (seconds since UNIX epoch).
    pub expires_at: u64,
    /// Relevant HTTP response headers stored so they can be replayed.
    pub response_headers: HashMap<String, String>,
}

// ── Cache index (persisted to disk) ─────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheIndex {
    /// URL → CacheEntry mapping.
    pub entries: HashMap<String, CacheEntry>,
    /// Total on-disk size of all cached files (bytes).
    pub total_size: u64,
    /// Cumulative cache-hit counter.
    pub hit_count: u64,
    /// Cumulative cache-miss counter.
    pub miss_count: u64,
    /// Timestamps of the last N cache hits (for hit-rate calculation).
    #[serde(default)]
    pub hit_timestamps: Vec<u64>,
}

impl Default for CacheIndex {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            total_size: 0,
            hit_count: 0,
            miss_count: 0,
            hit_timestamps: Vec::new(),
        }
    }
}

// ── Engine ──────────────────────────────────────────────────────

pub struct CacheEngine {
    /// In-memory index, persisted to cache-dir/cache-index.json.
    pub index: RwLock<CacheIndex>,
    /// Root directory: `$APP_DATA_DIR/cache/`.
    cache_dir: PathBuf,
    /// Cache size cap in bytes (uses AtomicU64 for runtime resize).
    max_size: AtomicU64,
    /// Counter for LRU ordering — incremented on every access.
    lru_counter: AtomicU64,
}

impl CacheEngine {
    /// Create or load the cache engine.
    pub fn new(cache_dir: PathBuf, max_size_mb: u32) -> Self {
        let max_size = (max_size_mb as u64) * 1024 * 1024;
        let _ = fs::create_dir_all(&cache_dir);

        let index_path = cache_dir.join("cache-index.json");
        let index = if index_path.exists() {
            fs::read_to_string(&index_path)
                .ok()
                .and_then(|json| serde_json::from_str(&json).ok())
                .unwrap_or_default()
        } else {
            CacheIndex::default()
        };

        let engine = Self {
            index: RwLock::new(index),
            cache_dir,
            max_size: AtomicU64::new(max_size),
            lru_counter: AtomicU64::new(0),
        };

        // Auto-evict if the persisted index already exceeds the limit.
        engine.maybe_evict();
        engine
    }

    // ── helpers ──────────────────────────────────────────────

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    fn hash_url(url: &str) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        url.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    fn index_path(&self) -> PathBuf {
        self.cache_dir.join("cache-index.json")
    }

    fn persist_index(&self) {
        let index = self.index.read().ok();
        if let Some(index) = index {
            let json = serde_json::to_string_pretty(&*index).unwrap_or_default();
            let _ = fs::write(self.index_path(), &json);
        }
    }

    fn next_lru(&self) -> u64 {
        self.lru_counter.fetch_add(1, Ordering::Relaxed)
    }

    // ── cache dir path ───────────────────────────────────────

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    // ── lookup ───────────────────────────────────────────────

    /// Check whether `url` is cached and the entry has not expired.
    /// Returns `Some(entry)` on hit and updates last_accessed, `None` on miss.
    pub fn check_cache(&self, url: &str) -> Option<CacheEntry> {
        let mut index = match self.index.write() {
            Ok(i) => i,
            Err(_) => return None,
        };

        let now = Self::now_secs();
        let expired = index.entries.get(url).map_or(false, |e| now > e.expires_at);

        if expired {
            if let Some(entry) = index.entries.remove(url) {
                let file_path = self.cache_dir.join(&entry.hash);
                let _ = fs::remove_file(&file_path);
                index.total_size = index.total_size.saturating_sub(entry.size);
            }
            drop(index);
            self.persist_index();
            return None;
        }

        // Not in cache at all.
        if !index.entries.contains_key(url) {
            return None;
        }

        // Cache hit — update stats.
        if let Some(entry) = index.entries.get_mut(url) {
            entry.last_accessed = now;
        }
        index.hit_count = index.hit_count.saturating_add(1);
        index.hit_timestamps.push(now);
        if index.hit_timestamps.len() > 1000 {
            let split_at = index.hit_timestamps.len() - 500;
            index.hit_timestamps = index.hit_timestamps.split_off(split_at);
        }
        let cloned = index.entries.get(url)?.clone();
        drop(index);
        self.persist_index();
        Some(cloned)
    }

    // ── store ────────────────────────────────────────────────

    /// Store a response body and metadata in the cache.
    /// Returns the created `CacheEntry`.
    pub fn store(
        &self,
        url: &str,
        body: &[u8],
        content_type: &str,
        max_age_secs: Option<u64>,
        response_headers: HashMap<String, String>,
    ) -> Option<CacheEntry> {
        let size = body.len() as u64;
        if size == 0 || size > MAX_SINGLE_FILE {
            return None;
        }

        let hash = Self::hash_url(url);
        let now = Self::now_secs();
        let ttl = max_age_secs.unwrap_or(DEFAULT_TTL_SECS);

        let entry = CacheEntry {
            url: url.to_string(),
            hash: hash.clone(),
            content_type: content_type.to_string(),
            cached_at: now,
            last_accessed: now,
            size,
            expires_at: now.saturating_add(ttl),
            response_headers,
        };

        // Write body to disk.
        let file_path = self.cache_dir.join(&hash);
        if let Err(e) = fs::File::create(&file_path).and_then(|mut f| f.write_all(body)) {
            eprintln!("[Pake Cache] Failed to write {}: {e}", file_path.display());
            return None;
        }

        // Update index.
        {
            let mut index = match self.index.write() {
                Ok(i) => i,
                Err(_) => {
                    let _ = fs::remove_file(&file_path);
                    return None;
                }
            };

            // If we already have this URL cached, remove old file.
            if let Some(old) = index.entries.remove(url) {
                let old_path = self.cache_dir.join(&old.hash);
                let _ = fs::remove_file(&old_path);
                index.total_size = index.total_size.saturating_sub(old.size);
            }

            index.total_size = index.total_size.saturating_add(size);
            index.entries.insert(url.to_string(), entry.clone());
        }

        self.persist_index();

        // Maybe evict if over limit.
        self.maybe_evict();

        Some(entry)
    }

    // ── read body from disk ──────────────────────────────────

    /// Read the cached file body for a given entry.
    pub fn read_body(&self, entry: &CacheEntry) -> Option<Vec<u8>> {
        let path = self.cache_dir.join(&entry.hash);
        fs::read(&path).ok()
    }

    // ── LRU eviction ─────────────────────────────────────────

    /// Calculate the 1-hour hit rate (0.0 – 1.0).
    pub fn hit_rate_1h(&self) -> f64 {
        let index = match self.index.read() {
            Ok(i) => i,
            Err(_) => return 0.0,
        };
        let now = Self::now_secs();
        let one_hour = 3600u64;
        let recent_hits = index
            .hit_timestamps
            .iter()
            .filter(|&&ts| now.saturating_sub(ts) <= one_hour)
            .count();
        let total = index.hit_timestamps.len().max(1);
        recent_hits as f64 / total as f64
    }

    fn maybe_evict(&self) {
        loop {
            let (total, max) = {
                let index = match self.index.read() {
                    Ok(i) => i,
                    Err(_) => return,
                };
                (index.total_size, self.max_size.load(Ordering::Relaxed))
            };
            if total <= max {
                break;
            }
            self.evict_one_chunk();
        }
    }

    fn evict_one_chunk(&self) {
        let max = self.max_size.load(Ordering::Relaxed);
        let target = (max as f64 * (1.0 - EVICT_FRACTION)) as u64;
        let min_target = max.saturating_sub(MIN_EVICT_SIZE);
        let target = target.min(min_target);

        let mut index = match self.index.write() {
            Ok(i) => i,
            Err(_) => return,
        };

        if index.total_size <= target || index.entries.is_empty() {
            return;
        }

        // Sort entries by last_accessed (oldest first).
        let mut sorted: Vec<_> = index.entries.values().cloned().collect();
        sorted.sort_by_key(|e| e.last_accessed);

        for entry in &sorted {
            if index.total_size <= target || index.entries.len() <= 1 {
                break;
            }
            let file_path = self.cache_dir.join(&entry.hash);
            let _ = fs::remove_file(&file_path);
            index.total_size = index.total_size.saturating_sub(entry.size);
            index.entries.remove(&entry.url);
        }
        drop(index);
        self.persist_index();
    }

    pub fn evict_for_url(&self, url: &str) {
        let mut index = match self.index.write() {
            Ok(i) => i,
            Err(_) => return,
        };
        if let Some(entry) = index.entries.remove(url) {
            let file_path = self.cache_dir.join(&entry.hash);
            let _ = fs::remove_file(&file_path);
            index.total_size = index.total_size.saturating_sub(entry.size);
        }
        drop(index);
        self.persist_index();
    }

    // ── clear all ────────────────────────────────────────────

    pub fn clear_all(&self) {
        let mut index = match self.index.write() {
            Ok(i) => i,
            Err(_) => return,
        };
        for entry in index.entries.values() {
            let path = self.cache_dir.join(&entry.hash);
            let _ = fs::remove_file(&path);
        }
        index.entries.clear();
        index.total_size = 0;
        index.hit_count = 0;
        index.miss_count = 0;
        index.hit_timestamps.clear();
        drop(index);
        // Also remove the index file.
        let _ = fs::remove_file(self.index_path());
    }

    // ── runtime resize ────────────────────────────────────────

    /// Update the maximum cache size at runtime. Triggers eviction if
    /// current usage exceeds the new limit.
    pub fn resize(&self, max_size_mb: u32) {
        let new_max = (max_size_mb as u64) * 1024 * 1024;
        self.max_size.store(new_max, Ordering::Relaxed);
        self.evict_to_target(new_max);
    }

    /// Evict entries until total_size <= target bytes.
    fn evict_to_target(&self, target: u64) {
        // Repeatedly evict until under target or only one entry remains.
        loop {
            let current = {
                let index = match self.index.read() {
                    Ok(i) => i,
                    Err(_) => return,
                };
                index.total_size
            };
            if current <= target {
                break;
            }
            // Calculate how much to evict.
            let excess = current.saturating_sub(target);
            let chunk = excess.max(MIN_EVICT_SIZE);

            let mut index = match self.index.write() {
                Ok(i) => i,
                Err(_) => return,
            };
            if index.total_size <= target || index.entries.len() <= 1 {
                break;
            }
            let mut sorted: Vec<_> = index.entries.values().cloned().collect();
            sorted.sort_by_key(|e| e.last_accessed);

            let mut evicted = 0u64;
            for entry in &sorted {
                if index.total_size <= target || index.entries.len() <= 1 || evicted >= chunk {
                    break;
                }
                let file_path = self.cache_dir.join(&entry.hash);
                let _ = fs::remove_file(&file_path);
                index.total_size = index.total_size.saturating_sub(entry.size);
                evicted += entry.size;
                index.entries.remove(&entry.url);
            }
            drop(index);
            self.persist_index();
        }
    }

    // ── stats ────────────────────────────────────────────────

    pub fn stats(&self) -> CacheStats {
        let index = match self.index.read() {
            Ok(i) => i,
            Err(_) => {
                return CacheStats::default();
            }
        };
        CacheStats {
            file_count: index.entries.len() as u64,
            total_size: index.total_size,
            max_size: self.max_size.load(Ordering::Relaxed),
            hit_count: index.hit_count,
            miss_count: index.miss_count,
            hit_rate_1h: self.hit_rate_1h(),
        }
    }

    pub fn stats_json(&self) -> serde_json::Value {
        let stats = self.stats();
        serde_json::json!({
            "fileCount": stats.file_count,
            "totalSize": stats.total_size,
            "totalSizeMB": format!("{:.1}", stats.total_size as f64 / (1024.0 * 1024.0)),
            "maxSize": stats.max_size,
            "maxSizeMB": stats.max_size / (1024 * 1024),
            "hitCount": stats.hit_count,
            "missCount": stats.miss_count,
            "hitRate1h": format!("{:.0}%", stats.hit_rate_1h * 100.0),
        })
    }

    pub fn record_miss(&self) {
        if let Ok(mut index) = self.index.write() {
            index.miss_count = index.miss_count.saturating_add(1);
        }
        self.persist_index();
    }
}

// ── Stats struct ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Default)]
pub struct CacheStats {
    pub file_count: u64,
    pub total_size: u64,
    pub max_size: u64,
    pub hit_count: u64,
    pub miss_count: u64,
    pub hit_rate_1h: f64,
}
