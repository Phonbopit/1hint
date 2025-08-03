use axum::{
    extract::{Query, State},
    http::{HeaderMap, Method, StatusCode},
    response::Json,
    routing::{get, post},
    Json as AxumJson, Router,
};
use chrono::{DateTime, Utc};
use ethers::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{sqlite::SqlitePool, Row};
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Instant;
use tauri::{Listener, Manager};
use tokio::sync::oneshot;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

struct AppState {
    proxy_running: bool,
    proxy_port: Option<u16>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    anvil_running: bool,
    anvil_port: Option<u16>,
    anvil_process: Option<Child>,
    db_pool: Option<SqlitePool>,
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
            db_pool: None,
        }
    }
}

impl Drop for AppState {
    fn drop(&mut self) {
        if let Some(mut process) = self.anvil_process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }

        let _ = Command::new("pkill").args(["-f", "anvil"]).output();
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiRequest {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub method: String,
    pub url: String,
    pub status: Option<i32>,
    pub duration_ms: Option<i64>,
    pub request_body: Option<String>,
    pub response_body: Option<String>,
    pub error: Option<String>,
}

async fn init_database() -> Result<SqlitePool, sqlx::Error> {
    let database_path = "requests.db";

    if let Some(parent) = std::path::Path::new(database_path).parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let pool = SqlitePool::connect(&format!("sqlite://{}?mode=rwc", database_path)).await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS requests (
            id TEXT PRIMARY KEY,
            timestamp DATETIME NOT NULL,
            method TEXT NOT NULL,
            url TEXT NOT NULL,
            status INTEGER,
            duration_ms INTEGER,
            request_body TEXT,
            response_body TEXT,
            error TEXT
        )
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

async fn log_request(
    pool: &SqlitePool,
    id: String,
    method: String,
    url: String,
    request_body: Option<String>,
    status: Option<i32>,
    response_body: Option<String>,
    duration_ms: Option<i64>,
    error: Option<String>,
) {
    let timestamp = Utc::now();

    let result = sqlx::query(
        r#"
        INSERT INTO requests (id, timestamp, method, url, status, duration_ms, request_body, response_body, error)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#,
    )
    .bind(&id)
    .bind(&timestamp)
    .bind(&method)
    .bind(&url)
    .bind(status)
    .bind(duration_ms)
    .bind(request_body)
    .bind(response_body)
    .bind(error)
    .execute(pool)
    .await;

    if let Err(e) = result {
        eprintln!("Failed to log request: {}", e);
    }
}

async fn proxy_get(
    State(pool): State<SqlitePool>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Result<Json<Value>, StatusCode> {
    let request_id = Uuid::new_v4().to_string();
    let start_time = Instant::now();

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
        .await;

    let duration_ms = start_time.elapsed().as_millis() as i64;

    match response {
        Ok(resp) => {
            let status = resp.status().as_u16() as i32;
            let data: Result<Value, _> = resp.json().await;

            match data {
                Ok(response_data) => {
                    let response_body = serde_json::to_string(&response_data).ok();
                    log_request(
                        &pool,
                        request_id,
                        "GET".to_string(),
                        url.clone(),
                        None,
                        Some(status),
                        response_body,
                        Some(duration_ms),
                        None,
                    )
                    .await;
                    Ok(Json(response_data))
                }
                Err(e) => {
                    log_request(
                        &pool,
                        request_id,
                        "GET".to_string(),
                        url.clone(),
                        None,
                        Some(status),
                        None,
                        Some(duration_ms),
                        Some(e.to_string()),
                    )
                    .await;
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            log_request(
                &pool,
                request_id,
                "GET".to_string(),
                url.clone(),
                None,
                None,
                None,
                Some(duration_ms),
                Some(e.to_string()),
            )
            .await;
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn proxy_post(
    State(pool): State<SqlitePool>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    AxumJson(payload): AxumJson<HashMap<String, Value>>,
) -> Result<Json<Value>, StatusCode> {
    let request_id = Uuid::new_v4().to_string();
    let start_time = Instant::now();

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

    let request_body = serde_json::to_string(body).ok();

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .header("Authorization", auth_header)
        .header("Content-Type", "application/json")
        .json(body)
        .send()
        .await;

    let duration_ms = start_time.elapsed().as_millis() as i64;

    match response {
        Ok(resp) => {
            let status = resp.status().as_u16() as i32;
            let data: Result<Value, _> = resp.json().await;

            match data {
                Ok(response_data) => {
                    let response_body = serde_json::to_string(&response_data).ok();
                    log_request(
                        &pool,
                        request_id,
                        "POST".to_string(),
                        url.clone(),
                        request_body,
                        Some(status),
                        response_body,
                        Some(duration_ms),
                        None,
                    )
                    .await;
                    Ok(Json(response_data))
                }
                Err(e) => {
                    log_request(
                        &pool,
                        request_id,
                        "POST".to_string(),
                        url.clone(),
                        request_body,
                        Some(status),
                        None,
                        Some(duration_ms),
                        Some(e.to_string()),
                    )
                    .await;
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            log_request(
                &pool,
                request_id,
                "POST".to_string(),
                url.clone(),
                request_body,
                None,
                None,
                Some(duration_ms),
                Some(e.to_string()),
            )
            .await;
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[tauri::command]
async fn log_api_request(
    state: tauri::State<'_, Mutex<AppState>>,
    id: String,
    method: String,
    url: String,
    request_body: Option<String>,
    status: Option<i32>,
    response_body: Option<String>,
    duration_ms: Option<i64>,
    error: Option<String>,
) -> Result<(), String> {
    let pool = {
        let app_state = state.lock().map_err(|e| e.to_string())?;
        match &app_state.db_pool {
            Some(pool) => pool.clone(),
            None => return Err("Database not initialized".to_string()),
        }
    };

    log_request(
        &pool,
        id,
        method,
        url,
        request_body,
        status,
        response_body,
        duration_ms,
        error,
    )
    .await;

    Ok(())
}

#[tauri::command]
async fn get_request_history(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<ApiRequest>, String> {
    let pool = {
        let app_state = state.lock().map_err(|e| e.to_string())?;
        match &app_state.db_pool {
            Some(pool) => pool.clone(),
            None => return Err("Database not initialized".to_string()),
        }
    };

    let rows = sqlx::query("SELECT * FROM requests ORDER BY timestamp DESC LIMIT 100")
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?;

    let requests: Vec<ApiRequest> = rows
        .into_iter()
        .map(|row| ApiRequest {
            id: row.get("id"),
            timestamp: row.get("timestamp"),
            method: row.get("method"),
            url: row.get("url"),
            status: row.get("status"),
            duration_ms: row.get("duration_ms"),
            request_body: row.get("request_body"),
            response_body: row.get("response_body"),
            error: row.get("error"),
        })
        .collect();

    Ok(requests)
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
    let existing_pool = {
        let app_state = state.lock().map_err(|e| e.to_string())?;
        if app_state.proxy_running {
            return Err("Proxy server is already running".to_string());
        }
        app_state.db_pool.clone()
    };

    let pool = if let Some(existing_pool) = existing_pool {
        existing_pool
    } else {
        let new_pool = init_database().await.map_err(|e| e.to_string())?;
        println!("Database initialized successfully");
        new_pool
    };

    let shutdown_rx = {
        let mut app_state = state.lock().map_err(|e| e.to_string())?;

        if app_state.db_pool.is_none() {
            app_state.db_pool = Some(pool.clone());
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
        .layer(cors)
        .with_state(pool);

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
            return Err(format!(
                "Port {} is already in use. Kill the existing process first.",
                port
            ));
        }
    }

    let mut cmd = Command::new("anvil");
    cmd.args(["--port", &port.to_string(), "--host", "0.0.0.0"]);

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
        return Err(format!(
            "Anvil process exited immediately with status: {}",
            exit_status
        ));
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
            stop_anvil_node,
            log_api_request,
            get_request_history
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let state_handle = handle.clone();

            tauri::async_runtime::spawn(async move {
                match init_database().await {
                    Ok(pool) => {
                        let state = state_handle.state::<Mutex<AppState>>();
                        if let Ok(mut app_state) = state.lock() {
                            app_state.db_pool = Some(pool);
                            println!("Database initialized successfully on app startup");
                        };
                    }
                    Err(e) => {
                        eprintln!("Failed to initialize database: {}", e);
                    }
                }
            });

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
