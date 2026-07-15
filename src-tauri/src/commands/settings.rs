//! Tauri commands for app settings / parameter management.

use crate::{db, settings};

/// Get a setting value by key.
#[tauri::command]
pub fn settings_get(key: String) -> Result<Option<String>, String> {
    let conn = db::open_readonly()?;
    settings::get(&conn, &key).map_err(|e| e.to_string())
}

/// Get a typed (JSON-deserialized) setting value.
#[tauri::command]
pub fn settings_get_typed(key: String) -> Result<serde_json::Value, String> {
    let conn = db::open_readonly()?;
    let raw = settings::get(&conn, &key)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Key '{key}' not found"))?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

/// Set a key-value setting.
#[tauri::command]
pub fn settings_set(key: String, value: String) -> Result<(), String> {
    let conn = db::open_readwrite()?;
    settings::set(&conn, &key, &value).map_err(|e| e.to_string())
}

/// Set a typed setting value (serializes to JSON).
#[tauri::command]
pub fn settings_set_typed(key: String, value: serde_json::Value) -> Result<(), String> {
    let conn = db::open_readwrite()?;
    let raw = serde_json::to_string(&value).map_err(|e| e.to_string())?;
    settings::set(&conn, &key, &raw).map_err(|e| e.to_string())
}

/// Delete a setting key.
#[tauri::command]
pub fn settings_delete(key: String) -> Result<(), String> {
    let conn = db::open_readwrite()?;
    settings::delete(&conn, &key).map_err(|e| e.to_string())
}

/// List all settings, optionally filtered by prefix.
#[tauri::command]
pub fn settings_list(prefix: Option<String>) -> Result<Vec<(String, String)>, String> {
    let conn = db::open_readonly()?;
    settings::list_prefix(&conn, prefix.as_deref()).map_err(|e| e.to_string())
}
