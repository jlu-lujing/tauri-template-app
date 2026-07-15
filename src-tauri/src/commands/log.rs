//! Tauri commands for system logging.

use tracing::{debug, error, info, warn};

/// Query recent log entries with optional level filter.
#[tauri::command]
pub fn log_query(limit: i64, level_filter: Option<String>) -> Result<Vec<crate::logging::LogEntry>, String> {
    crate::logging::query_logs(limit, level_filter.as_deref())
}

/// Get total count of stored log entries.
#[tauri::command]
pub fn log_count() -> Result<i64, String> {
    crate::logging::log_count()
}

/// Clear all stored log entries.
#[tauri::command]
pub fn log_clear() -> Result<(), String> {
    crate::logging::clear_logs()
}

// Convenience functions that Tauri handlers can call to emit log events.
/// Log a debug message (also records to SQLite via tracing layer).
pub fn log_debug(target: &str, msg: &str) {
    debug!(target, "{msg}");
}

/// Log an info message.
pub fn log_info(target: &str, msg: &str) {
    info!(target, "{msg}");
}

/// Log a warning message.
pub fn log_warn(target: &str, msg: &str) {
    warn!(target, "{msg}");
}

/// Log an error message.
pub fn log_error(target: &str, msg: &str) {
    error!(target, "{msg}");
}
