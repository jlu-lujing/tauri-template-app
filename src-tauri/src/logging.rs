//! System logging backed by SQLite.
//!
//! Writes each log entry to both the console (via tracing) and a `log_entries`
//! table so that the React frontend can query and display historical logs.
//!

use serde::{Deserialize, Serialize};
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter};

use crate::db;

/// Log level for stored entries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Level::Trace => write!(f, "TRACE"),
            Level::Debug => write!(f, "DEBUG"),
            Level::Info => write!(f, "INFO"),
            Level::Warn => write!(f, "WARN"),
            Level::Error => write!(f, "ERROR"),
        }
    }
}

impl From<&str> for Level {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "trace" => Level::Trace,
            "debug" => Level::Debug,
            "info" => Level::Info,
            "warn" => Level::Warn,
            "error" => Level::Error,
            _ => Level::Info,
        }
    }
}

/// A single log entry stored in SQLite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: i64,
    pub level: String,
    pub target: String,
    pub message: String,
    pub created_at: String,
}

/// Query recent log entries with optional level filter.
pub fn query_logs(limit: i64, level_filter: Option<&str>) -> Result<Vec<LogEntry>, String> {
    let conn = db::open_readonly()?;

    let sql = match level_filter {
        Some(_lvl) => format!(
            "SELECT id, level, target, message, created_at FROM log_entries WHERE level = ?1 ORDER BY id DESC LIMIT ?2"
        ),
        None => "SELECT id, level, target, message, created_at FROM log_entries ORDER BY id DESC LIMIT ?1".to_string(),
    };

    let mut stmt = conn.prepare(&sql)
        .map_err(|e| format!("Failed to prepare query: {e}"))?;

    let entries = if let Some(lvl) = level_filter {
        stmt.query_map((lvl, limit), |row| {
            Ok(LogEntry {
                id: row.get(0)?,
                level: row.get(1)?,
                target: row.get(2)?,
                message: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| format!("Failed to query logs: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read log rows: {e}"))?
    } else {
        stmt.query_map((&limit,), |row| {
            Ok(LogEntry {
                id: row.get(0)?,
                level: row.get(1)?,
                target: row.get(2)?,
                message: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| format!("Failed to query logs: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read log rows: {e}"))?
    };

    Ok(entries)
}

/// Insert a log entry into the SQLite table.
pub fn insert_log(level: &str, target: &str, message: &str) -> Result<(), String> {
    let conn = db::open_readwrite()?;
    conn.execute(
        "INSERT INTO log_entries (level, target, message) VALUES (?1, ?2, ?3)",
        [level, target, message],
    )
    .map_err(|e| format!("Failed to insert log: {e}"))?;
    Ok(())
}

/// Clear all stored log entries.
pub fn clear_logs() -> Result<(), String> {
    let conn = db::open_readwrite()?;
    conn.execute("DELETE FROM log_entries", [])
        .map_err(|e| format!("Failed to clear logs: {e}"))?;
    Ok(())
}

/// Get total count of stored log entries.
pub fn log_count() -> Result<i64, String> {
    let conn = db::open_readonly()?;
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM log_entries")
        .map_err(|e| format!("Failed to prepare count query: {e}"))?;
    let count: i64 = stmt.query_row([], |row| row.get(0))
        .map_err(|e| format!("Failed to get count: {e}"))?;
    Ok(count)
}

/// Initialize the global tracing subscriber.
///
/// - Reads `RUST_LOG` env var first; falls back to `cfg.level`.
/// - Bridges `log` crate macros into tracing.
pub fn init_logger(cfg: &LoggerConfig) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&cfg.level));

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_names(true)
        .with_file(false)
        .with_line_number(false);

    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer);

    tracing::subscriber::set_global_default(subscriber)
        .expect("failed to set tracing subscriber");

    // Bridge log crate -> tracing
    let _ = tracing_log::LogTracer::init();
}

/// Logger configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggerConfig {
    pub level: String,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
        }
    }
}
