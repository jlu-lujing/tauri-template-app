//! Database helpers — shared DB connection and migration logic.
//!
//! Since rusqlite::Connection is not Sync, we store only the database file
//! path in a global OnceLock and open fresh connections per-call. This avoids
//! thread-safety issues while keeping SQLite access efficient for read-heavy workloads.

use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::OnceLock;

static DB_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Store the resolved database path during initialization.
pub fn store_path(path: PathBuf) {
    let _ = DB_PATH.set(path);
}

/// Returns the path to the SQLite database file inside the app data directory.
pub fn db_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    use tauri::Manager;
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {e}"))?;
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data dir: {e}"))?;
    Ok(data_dir.join("app.db"))
}

/// Open a read-only connection.
pub fn open_readonly() -> Result<Connection, String> {
    let path = DB_PATH.get().ok_or("DB not initialized")?.clone();
    let conn = Connection::open(&path).map_err(|e| format!("Failed to open DB: {e}"))?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")
        .map_err(|e| format!("PRAGMA error: {e}"))?;
    Ok(conn)
}

/// Open a read-write connection.
pub fn open_readwrite() -> Result<Connection, String> {
    let path = DB_PATH.get().ok_or("DB not initialized")?.clone();
    let conn = Connection::open(&path).map_err(|e| format!("Failed to open DB: {e}"))?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")
        .map_err(|e| format!("PRAGMA error: {e}"))?;
    Ok(conn)
}

/// Open a connection and run migrations (called once at startup).
pub fn open_and_migrate(path: &PathBuf) -> Result<Connection, String> {
    let conn = Connection::open(path).map_err(|e| format!("Failed to open DB: {e}"))?;

    // Enable WAL mode for better concurrent read performance.
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA foreign_keys=ON;",
    )
    .map_err(|e| format!("Failed to set PRAGMAs: {e}"))?;

    // Create sys_config table (key-value settings).
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sys_config (
            key     TEXT PRIMARY KEY,
            value   TEXT NOT NULL DEFAULT '',
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )
    .map_err(|e| format!("Failed to create sys_config: {e}"))?;

    // Create log entries table for querying logs from the frontend.
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS log_entries (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            level     TEXT NOT NULL,
            target    TEXT DEFAULT '',
            message   TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );",
    )
    .map_err(|e| format!("Failed to create log_entries: {e}"))?;

    Ok(conn)
}
