//! Window control Tauri IPC commands

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, WindowEvent};

#[derive(Serialize, Deserialize)]
pub struct WindowState {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
}

#[tauri::command]
pub fn win_minimize(app: AppHandle) -> Result<(), String> {
    app.get_webview_window("main")
        .ok_or("no main window")?
        .minimize()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn win_maximize(app: AppHandle) -> Result<(), String> {
    let w = app.get_webview_window("main").ok_or("no main window")?;
    if w.is_maximized().unwrap_or(false) {
        w.unmaximize().map_err(|e| e.to_string())
    } else {
        w.maximize().map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn win_close(app: AppHandle) -> Result<(), String> {
    app.get_webview_window("main")
        .ok_or("no main window")?
        .close()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn win_platform() -> String {
    std::env::consts::OS.to_string()
}

#[tauri::command]
pub fn win_is_maximized(app: AppHandle) -> Result<bool, String> {
    app.get_webview_window("main")
        .ok_or("no main window")?
        .is_maximized()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn win_start_drag(app: AppHandle) -> Result<(), String> {
    app.get_webview_window("main")
        .ok_or("no main window")?
        .start_dragging()
        .map_err(|e| e.to_string())
}

/// Get current window state (position, size, maximized).
#[tauri::command]
pub fn win_get_state(app: AppHandle) -> Result<WindowState, String> {
    let w = app.get_webview_window("main").ok_or("no main window")?;

    let pos = match w.outer_position() {
        Ok(p) => (p.x, p.y),
        Err(_) => (0, 0),
    };

    let size = match w.outer_size() {
        Ok(s) => (s.width, s.height),
        Err(_) => (960, 680),
    };

    let maximized = w.is_maximized().unwrap_or(false);

    Ok(WindowState {
        x: pos.0,
        y: pos.1,
        width: size.0,
        height: size.1,
        maximized,
    })
}

/// Restore window from persisted state (position, size, maximized).
/// Called once during Tauri setup to position the window before it's shown.
pub fn restore_window_state(app: &tauri::AppHandle) {
    use tauri::{PhysicalPosition, PhysicalSize, Position, Size};

    let window = match app.get_webview_window("main") {
        Some(w) => w,
        None => return,
    };

    if let Some(state) = load_window_state(app) {
        // If stored state was maximized, unmaximize first to restore position/size
        if state.maximized {
            let _ = window.unmaximize();
        }

        // Resize so position is calculated relative to the correct size
        let _ = window.set_size(Size::Physical(PhysicalSize::<u32>::new(state.width, state.height)));

        // Restore stored position
        let _ = window.set_position(Position::Physical(
            PhysicalPosition::<i32>::new(state.x, state.y),
        ));
    }
}

/// Write current window state to disk.
#[tauri::command]
pub fn win_set_state(app: AppHandle, state: WindowState) -> Result<(), String> {
    save_window_state(&app, &state);
    Ok(())
}

/// Save current window state to disk.
pub fn save_window_state(app: &tauri::AppHandle, state: &WindowState) {
    let path = match app.path().app_data_dir() {
        Ok(dir) => dir.join("window_state.json"),
        Err(_) => return,
    };

    let data = match serde_json::to_string_pretty(state) {
        Ok(d) => d,
        Err(_) => return,
    };

    let _ = std::fs::write(&path, data);
}

/// Read persisted window state from disk.
fn load_window_state(app: &tauri::AppHandle) -> Option<WindowState> {
    let path = match app.path().app_data_dir() {
        Ok(dir) => dir.join("window_state.json"),
        Err(_) => return None,
    };

    let data = match std::fs::read_to_string(&path) {
        Ok(d) => d,
        Err(_) => return None,
    };

    serde_json::from_str(&data).ok()
}

/// Set up window event listeners: save state on resize/move and on close.
pub fn setup_window_save(app_handle: AppHandle) -> tauri::WebviewWindow {
    let window = app_handle.get_webview_window("main").expect("main window not found");

    // Save window state on resize/move (debounced by frontend timer)
    let app_clone = app_handle.clone();
    window.on_window_event(move |event| {
        if matches!(event, WindowEvent::Resized(_) | WindowEvent::Moved(_)) {
            let win = app_clone.get_webview_window("main");
            if let Some(w) = win {
                if let (Ok(size), Ok(pos)) = (w.outer_size(), w.outer_position()) {
                    let state = WindowState {
                        x: pos.x,
                        y: pos.y,
                        width: size.width,
                        height: size.height,
                        maximized: w.is_maximized().unwrap_or(false),
                    };
                    save_window_state(&app_clone, &state);
                }
            }
        }
    });

    // Save full window state on close
    let app_handle2 = app_handle.clone();
    let close_window = window.clone();
    close_window.clone().on_window_event(move |event| {
        if let WindowEvent::CloseRequested { .. } = event {
            let app = app_handle2.clone();
            let win = close_window.clone();

            // Save on main thread to ensure state is up-to-date
            let _ = app.run_on_main_thread({
                let app = app.clone();
                move || {
                    if let (Ok(size), Ok(pos)) = (win.outer_size(), win.outer_position()) {
                        let state = WindowState {
                            x: pos.x,
                            y: pos.y,
                            width: size.width,
                            height: size.height,
                            maximized: win.is_maximized().unwrap_or(false),
                        };
                        save_window_state(&app, &state);
                    }
                }
            });
        }
    });

    window
}
