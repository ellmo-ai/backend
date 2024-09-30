use axum::http::StatusCode;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use std::{collections::HashMap, sync::Arc};
use sync::Recipient::{Organization, User};
use sync::SyncPayload;
use tokio::sync::{mpsc, Mutex};

type SharedState = Arc<AppState>;

struct AppState {
    connections: Mutex<HashMap<i32, mpsc::UnboundedSender<Message>>>,
}

#[tokio::main]
async fn main() {
    let app_state = Arc::new(AppState {
        connections: Mutex::new(HashMap::new()),
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/ws", get(websocket_handler))
        .route("/send", post(send_message))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    println!(
        "HTTP server listening on {}",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

async fn websocket(stream: WebSocket, state: SharedState) {
    let (mut sink, mut stream) = stream.split();

    // We use a mpsc channel to send messages to the Sink from multiple threads
    let (sender, mut receiver) = mpsc::channel::<Message>(16);
    tokio::spawn(async move {
        while let Some(message) = receiver.recv().await {
            // Forward the message to the WebSocket sink
            if sink.send(message.into()).await.is_err() {
                break;
            }
        }
    });

    let client_id = 4;
    let (tx, mut rx) = mpsc::unbounded_channel();

    {
        let mut connections = state.connections.lock().await;
        connections.insert(client_id, tx);
    }

    let mut send_task = tokio::spawn(async move {
        while let Some(message) = stream.next().await {
            let message = match message {
                Ok(message) => message,
                Err(_) => break,
            };

            let message = match message {
                Message::Text(text) => text,
                Message::Binary(_) => continue,
                _ => "Unsupported message type".to_string(),
            };
        }
    });

    // Remove the connection from the HashMap
    let mut connections = state.connections.lock().await;
    connections.remove(&client_id);

    println!("WebSocket connection closed: {}", client_id);
}

async fn send_message(
    State(state): State<SharedState>,
    Json(payload): Json<SyncPayload>,
) -> impl IntoResponse {
    let recipient = payload.recipient;
    let messages = payload.messages;

    for message in messages {
        let connections = state.connections.lock().await;
        let connection = match recipient {
            Organization(id) => connections.get(&id),
            User(id) => connections.get(&id),
        };

        if let Some(connection) = connection {
            let message = serde_json::to_string(&message).unwrap();
            if connection.send(Message::Text(message)).is_err() {
                // Handle the error if needed
            }
        }
    }

    (StatusCode::OK, Json(()))
}
