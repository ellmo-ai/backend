use olly_db::{
    models::{repository::DieselRepository, test_registration::TestRegistration},
    schema::test_registration,
};

use aws_sdk_s3 as s3;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use tokio::{fs::File, io::AsyncWriteExt};
use tower_http::cors::CorsLayer;
use url::{Position, Url};

use flate2::read::GzDecoder;
use std::io::BufReader;
use tar::Archive;

#[derive(Clone)]
struct AppState {
    // TODO: Add S3 client
    db_pool: Arc<olly_db::DbPool>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_pool = Arc::new(olly_db::establish_connection_pool());

    let state = AppState { db_pool };

    let app = Router::new()
        .route("/", get(root))
        .route("/execute", post(execute))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    println!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await?;
    Ok(())
}

async fn root() -> &'static str {
    "Hello, World!"
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Test {
    name: String,
    version: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ExecuteRequestPayload {
    test: Test,
    input: String,
}

async fn execute(
    State(state): State<AppState>,
    Json(payload): Json<ExecuteRequestPayload>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut conn = state.db_pool.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get database connection: {}", e),
        )
    })?;

    let repo = DieselRepository {
        connection: &mut conn,
        table: test_registration::table,
    };

    let test_registration = repo
        .table
        .order_by(test_registration::created_at.desc())
        .first::<TestRegistration>(repo.connection)
        .optional()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch latest test registration: {}", e),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            "No test registration found".to_string(),
        ))?;

    let key = get_object_key(&test_registration.blob_url).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Failed to parse object key: {}", e),
        )
    })?;

    println!("Object key: {}", key);

    let file_path = format!("/tmp/{}", &key);

    // Only download the file if it doesn't exist
    if !std::path::Path::new(&file_path).exists() {
        println!("Downloading file from S3");

        let config = aws_config::load_from_env().await;
        let client = s3::Client::new(&config);

        let response = client
            .get_object()
            .bucket("test-registrations")
            .key(&key)
            .send()
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to download file: {}", e),
                )
            })?;

        println!("Writing file to disk");

        write_to_file(response.body, &file_path)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to write to file: {}", e),
                )
            })?;
    } else {
        println!("File already exists, skipping download");
    }

    execute_test(payload, "/tmp/extracted", test_registration).map_err(|e| {
        println!("Failed to execute test: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to execute test: {}", e),
        )
    })?;

    Ok(StatusCode::OK)
}

async fn write_to_file(
    mut stream: s3::primitives::ByteStream,
    file_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(file_path).await?;

    while let Some(bytes) = stream.try_next().await? {
        file.write_all(&bytes).await?;
    }
    file.flush().await?;

    extract_tar_gz(file_path, "/tmp/extracted")?;

    Ok(())
}

fn get_object_key(url: &str) -> Result<String, url::ParseError> {
    let parsed = Url::parse(url)?;
    let cleaned = &parsed[..Position::AfterPath];

    Ok(cleaned.split('/').last().unwrap_or("").to_string())
}

fn extract_tar_gz(tar_gz_path: &str, output_dir: &str) -> std::io::Result<()> {
    // Open the gzipped tarball
    let tar_gz = std::fs::File::open(tar_gz_path)?;
    let buf_reader = BufReader::new(tar_gz);

    // Create a GzDecoder around the tarball
    let tar = GzDecoder::new(buf_reader);

    // Extract the tarball
    let mut archive = Archive::new(tar);
    archive.unpack(output_dir)?;

    Ok(())
}

fn execute_test(
    request: ExecuteRequestPayload,
    path: &str,
    registration: TestRegistration,
) -> Result<(), Box<dyn std::error::Error>> {
    let test = request.test;
    let metadata = registration.metadata;

    let tests: olly_db::models::test_registration::Metadata = serde_json::from_value(metadata)?;

    let test_versions = tests.get(&test.name).ok_or("Test not found")?;
    let test = test_versions
        .iter()
        .find(|t| t.version == test.version)
        .ok_or("Test version not found")?;

    let file_path = format!("{}/{}", path, test.file_path);
    println!("Executing test {} at path {}", test.export_name, file_path);

    Ok(())
}
