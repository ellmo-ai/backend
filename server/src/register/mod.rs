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
    if prev_registration.is_none() {
        return Ok(true);
    }
    let prev_registration = prev_registration.unwrap();

    // Check the previous test registration to see if the tests are already registered
    let prev_metadata: Result<HashMap<TestId, Vec<Test>>, _> =
        serde_json::from_value(prev_registration.metadata);

    if prev_metadata.is_err() {
        return anyhow::Result::Err(anyhow::anyhow!("Failed to deserialize metadata"));
    }
    let prev_metadata = prev_metadata.unwrap();

    // We need to check if there are any new tests to register (either new test ids or new
    // version of existing test ids, or new export names/file paths)

    for (test_name, test_versions) in tests.iter() {
        let prev_test_versions = prev_metadata.get(test_name);
        if prev_test_versions.is_none() {
            // We are seeing a new test id
            return Ok(true);
        }

        // We have previous registrations for this test id
        let prev_test_versions = prev_test_versions.unwrap();

        for test_version in test_versions {
            match prev_test_versions
                .iter()
                .find(|x| x.version == test_version.version)
            {
                Some(existing_test_version) => {
                    // We have previous registrations for this test id and version
                    // Check if the export name and file path are the same
                    if existing_test_version.export_name != test_version.export_name
                        || existing_test_version.file_path != test_version.file_path
                    {
                        // We are seeing a new export name or file path for an existing test id
                        return Ok(true);
                    }
                }
                None => {
                    // We are seeing a new version of an existing test id
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}
