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
        .first::<TestRegistration>(repo.connection)
        .optional();

    if latest_test_registration.is_err() {
        println!(
            "Failed to fetch latest test registration, {}",
            latest_test_registration.err().unwrap()
        );
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(()));
    }

    let latest_test_registration = latest_test_registration.unwrap();

    match is_new_registration(latest_test_registration, &payload.tests) {
        Ok(true) => {
            // Continue with registering the new tests
        }
        Ok(false) => {
            // No new tests to register
            // TODO: return a message
            println!("No new tests to register");
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test(version: &str, export_name: &str, file_path: &str) -> Test {
        Test {
            version: version.to_string(),
            export_name: export_name.to_string(),
            file_path: file_path.to_string(),
        }
    }

    fn create_test_registration(meta: HashMap<TestId, Vec<Test>>) -> TestRegistration {
        TestRegistration {
            id: 1,
            metadata: serde_json::to_value(meta).unwrap(),
            blob_url: "https://example.com".to_string(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_no_previous_registration() {
        let tests = HashMap::new();
        assert!(is_new_registration(None, &tests).unwrap());
    }

    #[test]
    fn test_identical_registration() {
        let mut prev_metadata = HashMap::new();
        prev_metadata.insert(
            "test_id".to_string(),
            vec![create_test("1.0.0", "export_name", "file_path")],
        );

        let mut tests = HashMap::new();
        tests.insert(
            "test_id".to_string(),
            vec![create_test("1.0.0", "export_name", "file_path")],
        );

        let prev_registration = Some(create_test_registration(prev_metadata));
        assert!(!is_new_registration(prev_registration, &tests).unwrap());
    }

    #[test]
    fn test_new_test_id() {
        let prev_metadata = HashMap::new();

        let mut tests = HashMap::new();
        tests.insert(
            "new_test_id".to_string(),
            vec![create_test("1.0.0", "export_name", "file_path")],
        );

        let prev_registration = Some(create_test_registration(prev_metadata));
        assert!(is_new_registration(prev_registration, &tests).unwrap());
    }

    #[test]
    fn test_new_version() {
        let mut prev_metadata = HashMap::new();
        prev_metadata.insert(
            "test_id".to_string(),
            vec![create_test("1.0.0", "export_name", "file_path")],
        );

        let mut tests = HashMap::new();
        tests.insert(
            "test_id".to_string(),
            vec![
                create_test("1.0.0", "export_name", "file_path"),
                create_test("1.0.1", "export_name", "file_path"),
            ],
        );

        let prev_registration = Some(create_test_registration(prev_metadata));
        assert!(is_new_registration(prev_registration, &tests).unwrap());
    }

    #[test]
    fn test_changed_export_name() {
        let mut prev_metadata = HashMap::new();
        prev_metadata.insert(
            "test_id".to_string(),
            vec![create_test("1.0.0", "old_export_name", "file_path")],
        );

        let mut tests = HashMap::new();
        tests.insert(
            "test_id".to_string(),
            vec![create_test("1.0.0", "new_export_name", "file_path")],
        );

        let prev_registration = Some(create_test_registration(prev_metadata));
        assert!(is_new_registration(prev_registration, &tests).unwrap());
    }

    #[test]
    fn test_changed_file_path() {
        let mut prev_metadata = HashMap::new();
        prev_metadata.insert(
            "test_id".to_string(),
            vec![create_test("1.0.0", "export_name", "old_file_path")],
        );

        let mut tests = HashMap::new();
        tests.insert(
            "test_id".to_string(),
            vec![create_test("1.0.0", "export_name", "new_file_path")],
        );

        let prev_registration = Some(create_test_registration(prev_metadata));
        assert!(is_new_registration(prev_registration, &tests).unwrap());
    }

    #[test]
    fn test_invalid_metadata() {
        let invalid_metadata = serde_json::json!({"invalid": "data"});
        let prev_registration = Some(TestRegistration {
            id: 1,
            metadata: invalid_metadata,
            blob_url: "https://example.com".to_string(),
            created_at: Utc::now(),
        });

        let tests = HashMap::new();
        assert!(is_new_registration(prev_registration, &tests).is_err());
    }
}
