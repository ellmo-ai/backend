#![allow(dead_code)]

use serde::Serialize;

#[derive(Serialize)]
pub enum SyncType {
    Create,
    Update,
    Delete,
}

#[derive(Serialize)]
pub struct SyncMessage {
    pub sync_type: SyncType,
    pub payload: serde_json::Value,
}

/// Recipient of the sync message
#[derive(Serialize)]
pub enum Recipient {
    /// Send the message to all users in the organization
    Organization(i32),
    /// Send the message to a specific user
    User(i32),
}

#[derive(Serialize)]
pub struct SyncPayload {
    pub recipient: Recipient,
    pub messages: Vec<SyncMessage>,
}
