use std::sync::Mutex;
use std::collections::HashMap;
use tokio::sync::oneshot;
use axum::{
    extract::Query,
    http::{HeaderMap, Method, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
    Json as AxumJson,
};
use tower_http::cors::{CorsLayer, Any};
use serde_json::Value;

struct AppState {
    proxy_running: bool,
    proxy_port: Option<u16>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            proxy_running: false,
            proxy_port: None,
            shutdown_tx: None,
        }
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

    let data: Value = response.json().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
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

    let data: Value = response.json().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(data))
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn start_proxy_server(state: tauri::State<'_, Mutex<AppState>>, port: u16) -> Result<String, String> {
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
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH])
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            greet,
            start_proxy_server,
            stop_proxy_server
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}