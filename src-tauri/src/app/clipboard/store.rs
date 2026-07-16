use chrono::{Duration, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

const CLEANUP_BATCH_SIZE: i64 = 500;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClipboardEntry {
    pub id: i64,
    pub content: String,
    pub created_at: String,
    pub source_app: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClipboardStats {
    pub total: i64,
    pub max_items: u32,
}

pub struct ClipboardStore {
    conn: Mutex<Connection>,
    max_items: AtomicU32,
    retention_days: AtomicU32,
}

impl ClipboardStore {
    pub fn open(data_dir: &Path, max_items: u32, retention_days: u32) -> rusqlite::Result<Self> {
        let db_dir = data_dir.join("clipboard");
        std::fs::create_dir_all(&db_dir)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
        let conn = Connection::open(db_dir.join("history.db"))?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             CREATE TABLE IF NOT EXISTS clipboard_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                created_at TEXT NOT NULL,
                source_app TEXT
             );
             DELETE FROM clipboard_history
             WHERE id NOT IN (
                SELECT MAX(id) FROM clipboard_history GROUP BY content_hash
             );
             CREATE UNIQUE INDEX IF NOT EXISTS idx_clipboard_hash
                ON clipboard_history(content_hash);
             CREATE INDEX IF NOT EXISTS idx_clipboard_created_at
                ON clipboard_history(created_at);",
        )?;
        ensure_source_app_column(&conn)?;

        let store = Self {
            conn: Mutex::new(conn),
            max_items: AtomicU32::new(max_items),
            retention_days: AtomicU32::new(retention_days),
        };
        store.cleanup()?;
        Ok(store)
    }

    pub fn hash_content(text: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    #[cfg(test)]
    pub fn insert(&self, text: &str) -> rusqlite::Result<Option<i64>> {
        self.insert_with_source(text, None)
    }

    pub fn insert_with_source(
        &self,
        text: &str,
        source_app: Option<&str>,
    ) -> rusqlite::Result<Option<i64>> {
        let hash = Self::hash_content(text);
        let now = Utc::now().to_rfc3339();
        let conn = self.lock()?;
        let existing = conn
            .query_row(
                "SELECT id FROM clipboard_history WHERE content_hash = ?1",
                params![hash],
                |row| row.get::<_, i64>(0),
            )
            .optional()?;

        conn.execute(
            "INSERT INTO clipboard_history (content, content_hash, created_at, source_app)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(content_hash) DO UPDATE SET
                content = excluded.content,
                created_at = excluded.created_at,
                source_app = COALESCE(excluded.source_app, clipboard_history.source_app)",
            params![text, hash, now, source_app],
        )?;
        let id = existing.unwrap_or_else(|| conn.last_insert_rowid());
        drop(conn);
        self.cleanup()?;
        Ok(existing.is_none().then_some(id))
    }

    pub fn list(&self, limit: i64) -> rusqlite::Result<Vec<ClipboardEntry>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(
            "SELECT id, content, created_at, source_app
             FROM clipboard_history
             ORDER BY created_at DESC, id DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], entry_from_row)?;
        rows.collect()
    }

    pub fn search(&self, query: &str, limit: i64) -> rusqlite::Result<Vec<ClipboardEntry>> {
        let keywords: Vec<String> = query
            .split_whitespace()
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .map(|part| escape_like_pattern(&part.to_lowercase()))
            .collect();

        if keywords.is_empty() {
            return self.list(limit);
        }

        let mut sql = String::from(
            "SELECT id, content, created_at, source_app FROM clipboard_history WHERE ",
        );
        for index in 0..keywords.len() {
            if index > 0 {
                sql.push_str(" AND ");
            }
            sql.push_str(&format!("LOWER(content) LIKE ?{} ESCAPE '\\'", index + 1));
        }
        sql.push_str(&format!(
            " ORDER BY created_at DESC, id DESC LIMIT ?{}",
            keywords.len() + 1
        ));

        let patterns: Vec<String> = keywords.iter().map(|word| format!("%{word}%")).collect();
        let mut values: Vec<&dyn rusqlite::ToSql> = patterns
            .iter()
            .map(|pattern| pattern as &dyn rusqlite::ToSql)
            .collect();
        values.push(&limit);

        let conn = self.lock()?;
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(values), entry_from_row)?;
        rows.collect()
    }

    pub fn get(&self, id: i64) -> rusqlite::Result<Option<ClipboardEntry>> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT id, content, created_at, source_app FROM clipboard_history WHERE id = ?1",
            params![id],
            entry_from_row,
        )
        .optional()
    }

    pub fn delete(&self, id: i64) -> rusqlite::Result<bool> {
        let conn = self.lock()?;
        Ok(conn.execute("DELETE FROM clipboard_history WHERE id = ?1", params![id])? > 0)
    }

    pub fn clear_all(&self) -> rusqlite::Result<()> {
        let conn = self.lock()?;
        conn.execute("DELETE FROM clipboard_history", [])?;
        Ok(())
    }

    pub fn stats(&self) -> rusqlite::Result<ClipboardStats> {
        let conn = self.lock()?;
        let total = conn.query_row("SELECT COUNT(*) FROM clipboard_history", [], |row| {
            row.get(0)
        })?;
        Ok(ClipboardStats {
            total,
            max_items: self.max_items.load(Ordering::Relaxed),
        })
    }

    pub fn set_limits(&self, max_items: u32, retention_days: u32) -> rusqlite::Result<()> {
        self.max_items.store(max_items, Ordering::Relaxed);
        self.retention_days.store(retention_days, Ordering::Relaxed);
        self.cleanup()
    }

    pub fn cleanup(&self) -> rusqlite::Result<()> {
        let max_items = self.max_items.load(Ordering::Relaxed) as i64;
        let retention_days = self.retention_days.load(Ordering::Relaxed) as i64;
        let cutoff = (Utc::now() - Duration::days(retention_days)).to_rfc3339();
        let conn = self.lock()?;
        conn.execute(
            "DELETE FROM clipboard_history WHERE created_at < ?1",
            params![cutoff],
        )?;

        let total: i64 = conn.query_row("SELECT COUNT(*) FROM clipboard_history", [], |row| {
            row.get(0)
        })?;
        if total > max_items {
            let excess = total - max_items;
            let batch_count = (excess + CLEANUP_BATCH_SIZE - 1) / CLEANUP_BATCH_SIZE;
            let delete_count = (batch_count * CLEANUP_BATCH_SIZE).min(total);
            conn.execute(
                "DELETE FROM clipboard_history WHERE id IN (
                    SELECT id FROM clipboard_history
                    ORDER BY created_at ASC, id ASC
                    LIMIT ?1
                 )",
                params![delete_count],
            )?;
        }
        Ok(())
    }

    fn lock(&self) -> rusqlite::Result<std::sync::MutexGuard<'_, Connection>> {
        self.conn.lock().map_err(|_| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::other(
                "clipboard store lock poisoned",
            )))
        })
    }
}

fn ensure_source_app_column(conn: &Connection) -> rusqlite::Result<()> {
    let found = {
        let mut statement = conn.prepare("PRAGMA table_info(clipboard_history)")?;
        let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
        let mut found = false;
        for column in columns {
            if column?.eq_ignore_ascii_case("source_app") {
                found = true;
                break;
            }
        }
        found
    };
    if !found {
        conn.execute(
            "ALTER TABLE clipboard_history ADD COLUMN source_app TEXT",
            [],
        )?;
    }
    Ok(())
}

fn entry_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ClipboardEntry> {
    Ok(ClipboardEntry {
        id: row.get(0)?,
        content: row.get(1)?,
        created_at: row.get(2)?,
        source_app: row.get(3)?,
    })
}

fn escape_like_pattern(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_store(max_items: u32) -> (ClipboardStore, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "pake-clipboard-test-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        fs::create_dir_all(&dir).unwrap();
        let store = ClipboardStore::open(&dir, max_items, 30).unwrap();
        (store, dir)
    }

    fn close(store: ClipboardStore, dir: PathBuf) {
        drop(store);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn inserts_lists_and_deduplicates_entries() {
        let (store, dir) = temp_store(2_000);
        store.insert("first entry").unwrap();
        store.insert("second entry").unwrap();
        assert!(store.insert("first entry").unwrap().is_none());

        let items = store.list(10).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].content, "first entry");
        close(store, dir);
    }

    #[test]
    fn stores_and_updates_source_application() {
        let (store, dir) = temp_store(2_000);
        store
            .insert_with_source("source-aware entry", Some("notepad.exe"))
            .unwrap();
        assert_eq!(
            store.list(10).unwrap()[0].source_app.as_deref(),
            Some("notepad.exe")
        );

        store
            .insert_with_source("source-aware entry", Some("code.exe"))
            .unwrap();
        assert_eq!(
            store.list(10).unwrap()[0].source_app.as_deref(),
            Some("code.exe")
        );
        close(store, dir);
    }

    #[test]
    fn searches_with_chinese_english_and_literal_wildcards() {
        let (store, dir) = temp_store(2_000);
        store.insert("hello rust world").unwrap();
        store.insert("hello javascript").unwrap();
        store.insert("剪贴板 历史记录").unwrap();
        store.insert("100%_literal").unwrap();

        assert_eq!(store.search("hello rust", 10).unwrap().len(), 1);
        assert_eq!(store.search("剪贴板 历史", 10).unwrap().len(), 1);
        assert_eq!(store.search("%_", 10).unwrap().len(), 1);
        close(store, dir);
    }

    #[test]
    fn deletes_clears_and_reports_stats() {
        let (store, dir) = temp_store(2_000);
        let id = store.insert("one item").unwrap().unwrap();
        assert_eq!(store.stats().unwrap().total, 1);
        assert!(store.delete(id).unwrap());
        store.insert("another item").unwrap();
        store.clear_all().unwrap();
        assert_eq!(store.stats().unwrap().total, 0);
        close(store, dir);
    }

    #[test]
    fn removes_expired_entries() {
        let (store, dir) = temp_store(2_000);
        store.insert("expired item").unwrap();
        {
            let conn = store.lock().unwrap();
            let old = (Utc::now() - Duration::days(31)).to_rfc3339();
            conn.execute("UPDATE clipboard_history SET created_at = ?1", params![old])
                .unwrap();
        }
        store.cleanup().unwrap();
        assert_eq!(store.stats().unwrap().total, 0);
        close(store, dir);
    }

    #[test]
    fn trims_five_hundred_items_after_crossing_custom_capacity() {
        let (store, dir) = temp_store(500);
        for index in 0..501 {
            store.insert(&format!("entry number {index}")).unwrap();
        }
        assert_eq!(store.stats().unwrap().total, 1);
        close(store, dir);
    }

    #[test]
    fn trims_default_capacity_in_a_five_hundred_item_batch() {
        let (store, dir) = temp_store(2_000);
        for index in 0..2_001 {
            store
                .insert(&format!("default capacity entry {index}"))
                .unwrap();
        }
        assert_eq!(store.stats().unwrap().total, 1_501);
        close(store, dir);
    }
}
