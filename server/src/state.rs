use axum::extract::ws::Message;
use std::{collections::HashMap, sync::Mutex};
use tokio::sync::mpsc;
use uuid::Uuid;

pub struct AppState {
    // Map of connection ID to WebSocket sender
    pub connections: Mutex<HashMap<Uuid, mpsc::UnboundedSender<Message>>>,
}
