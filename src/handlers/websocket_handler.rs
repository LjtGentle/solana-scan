use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use serde_json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::services::websocket::WebSocketManager;

#[derive(serde::Deserialize)]
struct WebSocketMessage {
    action: String,
    address: Option<String>,
}

pub async fn start_websocket_server(ws_manager: Arc<RwLock<WebSocketManager>>) {
    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(ws_manager);

    let addr: std::net::SocketAddr = "0.0.0.0:8081".parse().unwrap();
    info!("WebSocket server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(ws_manager): State<Arc<RwLock<WebSocketManager>>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, ws_manager))
}

async fn handle_socket(socket: WebSocket, ws_manager: Arc<RwLock<WebSocketManager>>) {
    let connection_id = Uuid::new_v4().to_string();
    let (sender, mut receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    // 添加连接到管理器
    ws_manager
        .write()
        .await
        .add_connection(connection_id.clone(), tx.clone())
        .await;

    info!("WebSocket connection established: {}", connection_id);

    // 发送欢迎消息
    let welcome_msg = serde_json::json!({
        "type": "welcome",
        "connection_id": connection_id,
        "message": "Connected to Solana scanner WebSocket"
    });

    if tx.send(Message::Text(welcome_msg.to_string())).is_err() {
        error!("Failed to send welcome message to {}", connection_id);
        ws_manager
            .write()
            .await
            .remove_connection(&connection_id)
            .await;
        return;
    }

    // 从管理器转发消息到 websocket
    tokio::spawn(async move {
        let mut forward = sender;
        while let Some(msg) = rx.recv().await {
            if forward.send(msg).await.is_err() {
                break;
            }
        }
    });

    // 处理接收到的消息
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                debug!("Received message from {}: {}", connection_id, text);

                match serde_json::from_str::<WebSocketMessage>(&text) {
                    Ok(ws_msg) => {
                        handle_websocket_message(&ws_msg, &connection_id, ws_manager.clone()).await;
                    }
                    Err(e) => {
                        error!("Failed to parse WebSocket message: {}", e);
                        let error_msg = serde_json::json!({
                            "type": "error",
                            "message": "Invalid message format"
                        });
                        let _ = tx.send(Message::Text(error_msg.to_string()));
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("WebSocket connection closed: {}", connection_id);
                break;
            }
            Ok(_) => {
                // 忽略其他消息类型
            }
            Err(e) => {
                error!("WebSocket error for connection {}: {}", connection_id, e);
                break;
            }
        }
    }

    // 移除连接
    ws_manager
        .write()
        .await
        .remove_connection(&connection_id)
        .await;
    info!("WebSocket connection cleaned up: {}", connection_id);
}

async fn handle_websocket_message(
    msg: &WebSocketMessage,
    connection_id: &str,
    ws_manager: Arc<RwLock<WebSocketManager>>,
) {
    match msg.action.as_str() {
        "subscribe" => {
            if let Some(address) = &msg.address {
                let addr = address.clone();
                if let Err(e) = ws_manager
                    .write()
                    .await
                    .subscribe_to_address(&connection_id.to_string(), addr)
                    .await
                {
                    error!("Failed to subscribe to address: {}", e);
                }
            } else {
                error!("Subscribe action requires address");
            }
        }
        "unsubscribe" => {
            if let Some(address) = &msg.address {
                if let Err(e) = ws_manager
                    .write()
                    .await
                    .unsubscribe_from_address(&connection_id.to_string(), address)
                    .await
                {
                    error!("Failed to unsubscribe from address: {}", e);
                }
            } else {
                error!("Unsubscribe action requires address");
            }
        }
        _ => {
            error!("Unknown WebSocket action: {}", msg.action);
        }
    }
}
