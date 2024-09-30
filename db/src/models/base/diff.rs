#![allow(dead_code)]
use ellmo_sync::{SyncMessage, SyncPayload, SyncType};

use serde::Serialize;
use std::collections::BTreeMap;

pub type Diff = BTreeMap<String, DiffItem>;

#[derive(Serialize)]
pub struct DiffItem {
    pub(crate) before: serde_json::Value,
    pub(crate) after: serde_json::Value,
}

#[derive(Serialize)]
pub enum Change {
    Insert(serde_json::Value),
    Update(Diff),
    Delete(serde_json::Value),
}

pub trait Diffable: Serialize + Clone {
    fn diff(&self, other: &Self) -> Option<Diff>;
}

impl Change {
    pub async fn sync(&self) {
        let (sync_type, payload) = match self {
            Change::Insert(payload) => (SyncType::Create, payload.clone()),
            Change::Update(payload) => (SyncType::Update, serde_json::to_value(payload).unwrap()),
            Change::Delete(payload) => (SyncType::Delete, payload.clone()),
        };

        let payload = SyncPayload {
            organization_id: 1,
            messages: vec![SyncMessage { sync_type, payload }],
        };

        // Send payload to the server
        let res = reqwest::Client::new()
            .post("http://localhost:8080/sync")
            .json(&payload)
            .send();

        // Handle response
        match res.await {
            Ok(_) => println!("Sync successful"),
            Err(e) => eprintln!("Sync failed: {}", e),
        }
    }
}
