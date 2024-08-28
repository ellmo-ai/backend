use crate::db;
use diesel::prelude::*;

use axum::response::IntoResponse;
use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use db::models::{
    repository::{DieselRepository, Repository},
    test_registration::{InsertableTestRegistration, TestRegistration},
};
use db::schema::test_registration::dsl::*;

type TestId = String;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegisterTestPayload {
    tests: HashMap<TestId, Vec<Test>>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Test {
    version: String,
    export_name: String,
    file_path: String,
}

pub async fn test_post((Json(payload),): (Json<RegisterTestPayload>,)) -> impl IntoResponse {
    let mut conn = db::establish_connection();
    let mut repo = DieselRepository {
        connection: &mut conn,
        table: db::schema::test_registration::table,
    };

    let latest_test_registration = repo
        .table
        .order_by(created_at.desc())
        .first::<TestRegistration>(repo.connection);

    if latest_test_registration.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(()));
    }

    match is_new_registration(latest_test_registration.ok(), &payload.tests) {
        Ok(true) => {
            // Continue with registering the new tests
        }
        Ok(false) => {
            // No new tests to register
            // TODO: return a message
            return (StatusCode::OK, Json(()));
        }
        Err(e) => {
            println!("Failed checking registration, {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(()));
        }
    }

    let res = repo.create(&InsertableTestRegistration {
        metadata: serde_json::to_value(&payload.tests).unwrap(),
        blob_url: "https://example.com".to_string(),
        created_at: chrono::Utc::now(),
    });

    if res.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(()));
    }

    (StatusCode::OK, Json(()))
}

fn is_new_registration(
    prev_registration: Option<TestRegistration>,
    tests: &HashMap<TestId, Vec<Test>>,
) -> anyhow::Result<bool> {
    let prev_metadata = match prev_registration {
        Some(reg) => serde_json::from_value::<HashMap<TestId, Vec<Test>>>(reg.metadata)
            .map_err(|_| anyhow::anyhow!("Failed to deserialize metadata"))?,
        None => return Ok(true),
    };

    for (test_id, test_versions) in tests {
        let prev_versions = prev_metadata.get(test_id);

        if prev_versions.is_none() {
            return Ok(true); // New test ID
        }

        for test in test_versions {
            if !prev_versions.unwrap().iter().any(|prev| {
                prev.version == test.version
                    && prev.export_name == test.export_name
                    && prev.file_path == test.file_path
            }) {
                return Ok(true); // New version or changed export name/file path
            }
        }
    }

    Ok(false)
}
