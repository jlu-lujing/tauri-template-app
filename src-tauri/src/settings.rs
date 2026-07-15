//! Key-value settings store backed by SQLite.
//!
//! All values are stored as strings; callers handle type-specific deserialization.

use rusqlite::Connection;

/// Error type for settings operations.
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("database error: {0}")]
    Db(String),
    #[error("key not found: {0}")]
    NotFound(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl From<rusqlite::Error> for SettingsError {
    fn from(e: rusqlite::Error) -> Self {
        SettingsError::Db(e.to_string())
    }
}

/// Get a string value by key.
pub fn get(conn: &Connection, key: &str) -> Result<Option<String>, SettingsError> {
    let mut stmt = conn.prepare("SELECT value FROM sys_config WHERE key = ?1")?;
    let result = stmt.query_row([key], |row| row.get(0));
    match result {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(SettingsError::Db(e.to_string())),
    }
}

/// Set a key-value pair. Updates `updated_at` automatically via trigger logic.
pub fn set(conn: &Connection, key: &str, value: &str) -> Result<(), SettingsError> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO sys_config (key, value, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = ?3",
        [key, value, &now],
    )?;
    Ok(())
}

/// Delete a key.
pub fn delete(conn: &Connection, key: &str) -> Result<(), SettingsError> {
    conn.execute("DELETE FROM sys_config WHERE key = ?1", [key])?;
    Ok(())
}

/// List all settings, optionally filtered by a key prefix.
pub fn list_prefix(conn: &Connection, prefix: Option<&str>) -> Result<Vec<(String, String)>, SettingsError> {
    let pattern = match prefix {
        Some(p) => format!("{p}%"),
        None => "%".to_string(),
    };
    let mut stmt = conn.prepare("SELECT key, value FROM sys_config WHERE key LIKE ?1 ORDER BY key")?;
    let rows = stmt.query_map((&pattern,), |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Get a typed value (deserialized from JSON string).
pub fn get_typed<T: serde::de::DeserializeOwned>(conn: &Connection, key: &str) -> Result<T, SettingsError> {
    let raw = get(conn, key)?
        .ok_or_else(|| SettingsError::NotFound(key.to_string()))?;
    Ok(serde_json::from_str(&raw)?)
}

/// Set a typed value (serializes to JSON string).
pub fn set_typed<T: serde::Serialize>(conn: &Connection, key: &str, value: &T) -> Result<(), SettingsError> {
    let raw = serde_json::to_string(value)?;
    set(conn, key, &raw)?;
    Ok(())
}

// ── Well-known setting keys ────────────────────────────────────────────────

pub mod keys {
    // Display / UI settings (JSON blob)
    pub const UI_THEME: &str = "ui.theme";       // "dark" | "light"
    pub const UI_FONT_SIZE: &str = "ui.font_size"; // integer

    // Logging settings
    pub const LOG_LEVEL: &str = "log.level";      // "debug" | "info" | "warn" | "error"
}
