#![allow(dead_code)]

pub enum SyncType {
    Create,
    Update,
    Delete,
}

pub struct SyncMessage {
    pub sync_type: SyncType,
    pub payload: serde_json::Value,
}

pub struct SyncPayload {
    /// Organization ID, used to route the sync payload to the correct users
    pub organization_id: i32,
    pub messages: Vec<SyncMessage>,
}

/// Send a sync payload to the sync server
pub fn send_sync_payload(_payload: SyncPayload) {
    // Send payload to sync server
}
