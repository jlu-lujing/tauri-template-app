//! Tauri Template App — Generic desktop application template
//!
//! This crate provides the IPC layer between the React frontend and Tauri backend.

pub mod commands;
pub mod db;
pub mod logging;
pub mod settings;

use tauri::{WindowEvent, Manager};

/// Application entry point — called from main.rs
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // Window management
            commands::window::win_minimize,
            commands::window::win_maximize,
            commands::window::win_close,
            commands::window::win_start_drag,
            commands::window::win_platform,
            commands::window::win_is_maximized,
            commands::window::win_get_state,
            commands::window::win_set_state,
            // System logging
            commands::log::log_query,
            commands::log::log_count,
            commands::log::log_clear,
            // Settings / parameters
            commands::settings::settings_get,
            commands::settings::settings_get_typed,
            commands::settings::settings_set,
            commands::settings::settings_set_typed,
            commands::settings::settings_delete,
            commands::settings::settings_list,
        ])
        .setup(|app| {
            // ── Initialize database & logger ───────────────────────────────
            let db_path = db::db_path(app.handle())?;
            let conn = db::open_and_migrate(&db_path)?;
            db::store_path(db_path.clone());
            drop(conn); // We only need the path; fresh connections per-call.

            // Load log level from settings (or use default)
            let mut log_level = "info".to_string();
            {
                let conn = db::open_readonly()?;
                if let Ok(Some(val)) = settings::get(&conn, "log.level") {
                    log_level = val;
                }
            }

            logging::init_logger(&logging::LoggerConfig { level: log_level });

            // macOS: enable window shadow for borderless transparent window
            #[cfg(target_os = "macos")]
            {
                use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState};

                let window = app.get_webview_window("main").expect("main window not found");
                let _ = window.set_shadow(true);

                // Apply native macOS vibrancy (NSVisualEffectView) — blurs the
                // desktop wallpaper through the window. The webview stays
                // transparent so the vibrancy shows through.
                apply_vibrancy(
                    &window,
                    NSVisualEffectMaterial::HudWindow,
                    Some(NSVisualEffectState::Active),
                    None,
                )
                .expect("failed to apply vibrancy");

                // macOS: fix focus loss on frameless window when switching back
                let w = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::Focused(true) = event {
                        let _ = w.set_focus();
                    }
                });

                // macOS: delayed activation on first launch
                let w0 = window.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    let _ = w0.set_focus();
                });

                // ── Restore persisted window state (position, size, maximized) ─
                commands::window::restore_window_state(app.handle());

                // ── Set up window save listeners ─────────────────────────────
                let _ = commands::window::setup_window_save(app.handle().clone());
            }

            #[cfg(not(target_os = "macos"))]
            {
                // Non-macOS: simple restore + save setup without vibrancy
                let _window = app.get_webview_window("main").expect("main window not found");
                commands::window::restore_window_state(app.handle());
                let _ = commands::window::setup_window_save(app.handle().clone());
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
