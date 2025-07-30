use std::sync::Mutex;

// State for storing API key and proxy server status
struct AppState {
    api_key: Option<String>,
    proxy_running: bool,
    proxy_port: Option<u16>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            api_key: None,
            proxy_running: false,
            proxy_port: None,
        }
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn store_api_key(state: tauri::State<Mutex<AppState>>, key: String) -> Result<(), String> {
    let mut app_state = state.lock().map_err(|e| e.to_string())?;
    app_state.api_key = Some(key);
    Ok(())
}

#[tauri::command]
fn test_api_key(state: tauri::State<Mutex<AppState>>) -> Result<bool, String> {
    let app_state = state.lock().map_err(|e| e.to_string())?;
    match &app_state.api_key {
        Some(key) => {
            // TODO: test
            Ok(!key.trim().is_empty())
        }
        None => Ok(false),
    }
}

#[tauri::command]
fn start_proxy_server(state: tauri::State<Mutex<AppState>>, port: u16) -> Result<String, String> {
    let mut app_state = state.lock().map_err(|e| e.to_string())?;

    if app_state.proxy_running {
        return Err("Proxy server is already running".to_string());
    }

    // Mock proxy server start - in real implementation, you'd start actual proxy
    app_state.proxy_running = true;
    app_state.proxy_port = Some(port);

    let proxy_url = format!("http://localhost:{}", port);
    Ok(proxy_url)
}

#[tauri::command]
fn stop_proxy_server(state: tauri::State<Mutex<AppState>>) -> Result<(), String> {
    let mut app_state = state.lock().map_err(|e| e.to_string())?;

    if !app_state.proxy_running {
        return Err("Proxy server is not running".to_string());
    }

    // Mock proxy server stop - in real implementation, you'd stop actual proxy
    app_state.proxy_running = false;
    app_state.proxy_port = None;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            greet,
            store_api_key,
            test_api_key,
            start_proxy_server,
            stop_proxy_server
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
