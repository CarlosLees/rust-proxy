use std::cell::RefCell;
use std::sync::{Arc};

use axum::{Extension, Json, Router, serve};
use axum::extract::WebSocketUpgrade;
use axum::extract::ws::{Message, WebSocket};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, Mutex};
use tokio::sync::broadcast::Receiver;

const SOCKET_ADDR: &str = "127.0.0.1:8080";

#[derive(Deserialize, Serialize)]
struct MessagePayload {
    content: String,
}

type Tx = broadcast::Sender<String>;

#[derive(Clone)]
struct AppState {
    tx: Tx,
    rx: Arc<Mutex<Receiver<String>>>
}

#[tokio::main]
async fn main() {
    // 创建广播通道
    let (tx, rx) = broadcast::channel(100);

    let app_state = Arc::new(AppState {
        tx,
        rx: Arc::new(Mutex::new(rx))
    });

    let app = Router::new()
        .route("/api", post(handle_http_connection))
        .route("/wx",get(ws_handler))
        .layer(Extension(app_state));

    let listener = TcpListener::bind(SOCKET_ADDR).await.unwrap();
    serve(listener, app).await.unwrap()
}

async fn ws_handler(ws: WebSocketUpgrade, Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

async fn handle_websocket(mut socket: WebSocket, state: Arc<AppState>) {
    while let Ok(message) = state.rx.lock().await.recv().await {
        println!("message:{:?}",message);
        socket.send(Message::Text(message)).await.unwrap();
    }
}

async fn handle_http_connection(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<MessagePayload>,
) -> impl IntoResponse {
    state.clone().tx.send(payload.content).expect("message send error");

    Json(String::from("ok"))
}
