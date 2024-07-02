use std::sync::{Arc};

use axum::{Extension, Json, Router, serve};
use axum::body::{Body, to_bytes};
use axum::extract::{Request, WebSocketUpgrade};
use axum::extract::ws::{Message, WebSocket};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use bytes::Bytes;
use tokio::net::TcpListener;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, Mutex};

const SOCKET_ADDR: &str = "127.0.0.1:8080";

type Tx = Sender<Arc<Box<Bytes>>>;

#[derive(Clone)]
struct AppState {
    tx: Tx,
    rx: Arc<Mutex<Receiver<Arc<Box<Bytes>>>>>
}

#[tokio::main]
async fn main() {
    // 创建广播通道
    let (tx, rx) = mpsc::channel::<Arc<Box<Bytes>>>(6553500);
    let app_state = Arc::new(AppState {
        tx,
        rx: Arc::new(Mutex::new(rx))
    });

    let app = Router::new()
        .route("/api", post(handle_http_connection))
        .route("/ws",get(ws_handler))
        .layer(Extension(app_state));

    let listener = TcpListener::bind(SOCKET_ADDR).await.unwrap();
    serve(listener, app).await.unwrap()
}

async fn ws_handler(ws: WebSocketUpgrade, Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

async fn handle_websocket(mut socket: WebSocket, state: Arc<AppState>) {
    while let Some(message) = state.rx.lock().await.recv().await {
        println!("message length:{:?}",message.len());
        socket.send(Message::Binary(message.to_vec())).await.unwrap();
    }
}

async fn handle_http_connection(
    Extension(state): Extension<Arc<AppState>>,
    request: Request<Body>
) -> impl IntoResponse {

    let method = request.method().clone();
    let uri = request.uri().clone();
    let headers = request.headers().clone();

    let content_length = headers.get("Content-Length").map_or("0",|res| {
        return res.to_str().unwrap();
    });

    // println!("content-length:{:?}",content_length);

    // 判断请求头的格式
    let content_type = headers.get("Content-Type").unwrap().to_str().unwrap();
    println!("content_type:{:?}",content_type);

    // 获取请求体
    let body_bytes = to_bytes(request.into_body(), content_length.parse().unwrap()).await.unwrap();
    // let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();
    let x = Arc::new(Box::new(body_bytes));

    // let result = format!(
    //     "Method: {}\nURI: {}\nHeaders: {:?}\nBody: {:?}",
    //     method, uri, headers, body_bytes
    // );
    println!("{:?}",x.clone().len());
    state.clone().tx.send(x.clone()).await.expect("message send error");
    Json(String::from("ok"))
}
