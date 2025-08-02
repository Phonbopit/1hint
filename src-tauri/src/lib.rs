use axum::{
    extract::Query,
    http::{HeaderMap, Method, StatusCode},
    response::Json,
    routing::{get, post},
    Json as AxumJson, Router,
};
use ethers::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use tauri::{Listener, Manager};
use tokio::sync::oneshot;
use tower_http::cors::{Any, CorsLayer};


struct AppState {
    proxy_running: bool,
    proxy_port: Option<u16>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    anvil_running: bool,
    anvil_port: Option<u16>,
    anvil_process: Option<Child>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            proxy_running: false,
            proxy_port: None,
            shutdown_tx: None,
            anvil_running: false,
            anvil_port: None,
            anvil_process: None,
        }
    }
}

impl Drop for AppState {
    fn drop(&mut self) {
        if let Some(mut process) = self.anvil_process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
        
        let _ = Command::new("pkill")
            .args(["-f", "anvil"])
            .output();
    }
}

async fn proxy_get(
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    let url = match params.get("url") {
        Some(url) => url,
        None => return Err(StatusCode::BAD_REQUEST),
    };

    if !url.starts_with("https://api.1inch.dev") {
        return Err(StatusCode::BAD_REQUEST);
    }

    let auth_header = match headers.get("authorization") {
        Some(header) => header.to_str().map_err(|_| StatusCode::BAD_REQUEST)?,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("Authorization", auth_header)
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let data: Value = response
        .json()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(data))
}

async fn proxy_post(
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    AxumJson(payload): AxumJson<HashMap<String, Value>>,
) -> Result<Json<Value>, StatusCode> {
    let url = match params.get("url") {
        Some(url) => url,
        None => return Err(StatusCode::BAD_REQUEST),
    };

    if !url.starts_with("https://api.1inch.dev") {
        return Err(StatusCode::BAD_REQUEST);
    }

    let auth_header = match headers.get("authorization") {
        Some(header) => header.to_str().map_err(|_| StatusCode::BAD_REQUEST)?,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let body = match payload.get("data") {
        Some(data) => data,
        None => return Err(StatusCode::BAD_REQUEST),
    };

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .header("Authorization", auth_header)
        .header("Content-Type", "application/json")
        .json(body)
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let data: Value = response
        .json()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(data))
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeStatus {
    pub is_running: bool,
    pub url: Option<String>,
    pub block_number: Option<u64>,
    pub chain_id: Option<u64>,
    pub gas_price: Option<String>,
}

#[tauri::command]
fn test_anvil() -> String {
    "Anvil test command working!".to_string()
}

#[tauri::command]
async fn get_node_status(rpc_url: String) -> Result<NodeStatus, String> {
    let provider = Provider::<Http>::try_from(&rpc_url)
        .map_err(|e| format!("Failed to connect to node: {}", e))?;

    let block_number = provider
        .get_block_number()
        .await
        .map_err(|e| format!("Failed to get block number: {}", e))?;

    let chain_id = provider
        .get_chainid()
        .await
        .map_err(|e| format!("Failed to get chain ID: {}", e))?;

    let gas_price = provider
        .get_gas_price()
        .await
        .map_err(|e| format!("Failed to get gas price: {}", e))?;

    Ok(NodeStatus {
        is_running: true,
        url: Some(rpc_url),
        block_number: Some(block_number.as_u64()),
        chain_id: Some(chain_id.as_u64()),
        gas_price: Some(ethers::utils::format_ether(gas_price)),
    })
}

#[tauri::command]
async fn start_proxy_server(
    state: tauri::State<'_, Mutex<AppState>>,
    port: u16,
) -> Result<String, String> {
    let shutdown_rx = {
        let mut app_state = state.lock().map_err(|e| e.to_string())?;

        if app_state.proxy_running {
            return Err("Proxy server is already running".to_string());
        }

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        app_state.shutdown_tx = Some(shutdown_tx);
        app_state.proxy_running = true;
        app_state.proxy_port = Some(port);

        shutdown_rx
    };

    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
        ])
        .allow_headers(Any)
        .allow_origin(Any);

    let app = Router::new()
        .route("/", get(proxy_get))
        .route("/", post(proxy_post))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .map_err(|e| format!("Failed to bind to port {}: {}", port, e))?;

    tokio::spawn(async move {
        let server = axum::serve(listener, app).with_graceful_shutdown(async {
            shutdown_rx.await.ok();
        });

        if let Err(e) = server.await {
            eprintln!("Server error: {}", e);
        }
    });

    Ok(format!("http://localhost:{}", port))
}

#[tauri::command]
fn stop_proxy_server(state: tauri::State<Mutex<AppState>>) -> Result<(), String> {
    let mut app_state = state.lock().map_err(|e| e.to_string())?;

    if !app_state.proxy_running {
        return Err("Proxy server is not running".to_string());
    }

    if let Some(shutdown_tx) = app_state.shutdown_tx.take() {
        let _ = shutdown_tx.send(());
    }

    app_state.proxy_running = false;
    app_state.proxy_port = None;

    Ok(())
}


#[tauri::command]
fn start_anvil_node(
    state: tauri::State<'_, Mutex<AppState>>,
    port: u16,
    chain_id: Option<u64>,
) -> Result<String, String> {
    let mut app_state = state.lock().map_err(|e| e.to_string())?;

    if app_state.anvil_running {
        return Err("Anvil node is already running".to_string());
    }

    let port_check = std::process::Command::new("lsof")
        .args(["-ti", &format!(":{}", port)])
        .output();
    
    if let Ok(output) = port_check {
        if !output.stdout.is_empty() {
            return Err(format!("Port {} is already in use. Kill the existing process first.", port));
        }
    }

    let mut cmd = Command::new("anvil");
    cmd.args([
        "--port",
        &port.to_string(),
        "--host",
        "0.0.0.0",
    ]);

    if let Some(chain_id) = chain_id {
        cmd.args(["--chain-id", &chain_id.to_string()]);
    }

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        format!(
            "Failed to start anvil: {}. Make sure anvil is installed.",
            e
        )
    })?;

    std::thread::sleep(std::time::Duration::from_millis(1000));

    if let Ok(Some(exit_status)) = child.try_wait() {
        return Err(format!("Anvil process exited immediately with status: {}", exit_status));
    }

    app_state.anvil_running = true;
    app_state.anvil_port = Some(port);
    app_state.anvil_process = Some(child);

    let url = format!("http://localhost:{}", port);
    Ok(url)
}

#[tauri::command]
fn stop_anvil_node(state: tauri::State<Mutex<AppState>>) -> Result<(), String> {
    let mut app_state = state.lock().map_err(|e| e.to_string())?;

    if let Some(mut process) = app_state.anvil_process.take() {
        let _ = process.kill();
        let _ = process.wait();
    }

    let output = Command::new("pkill")
        .args(["-f", "anvil"])
        .output()
        .map_err(|e| format!("Failed to kill anvil processes: {}", e))?;

    if !output.status.success() && output.status.code() != Some(1) {
        return Err("Failed to stop anvil processes".to_string());
    }

    std::thread::sleep(std::time::Duration::from_millis(500));

    app_state.anvil_running = false;
    app_state.anvil_port = None;

    Ok(())
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            greet,
            test_anvil,
            get_node_status,
            start_proxy_server,
            stop_proxy_server,
            start_anvil_node,
            stop_anvil_node
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            
            app.listen("tauri://close-requested", move |_| {
                let state = handle.state::<Mutex<AppState>>();
                if let Ok(mut app_state) = state.lock() {
                    if let Some(mut process) = app_state.anvil_process.take() {
                        let _ = process.kill();
                        let _ = process.wait();
                    }
                }
                let _ = Command::new("pkill").args(["-f", "anvil"]).output();
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
