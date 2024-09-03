use aws_sdk_s3 as s3;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use diesel::prelude::*;
use olly_db::{
    models::{repository::DieselRepository, test_registration::TestRegistration},
    schema::test_registration,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::{fs::File, io::AsyncWriteExt};
use tower_http::cors::CorsLayer;
use url::{Position, Url};

#[derive(Clone)]
struct AppState {
    s3_client: Arc<s3::Client>,
    db_pool: Arc<olly_db::DbPool>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[allow(deprecated)]
    let config = aws_config::load_from_env().await;

    let s3_client = Arc::new(s3::Client::new(&config));
    let db_pool = Arc::new(olly_db::establish_connection_pool());

    let state = AppState { s3_client, db_pool };

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
    println!("Received request: {:?}", payload);

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

    let response = state
        .s3_client
        .get_object()
        .bucket("test-registrations")
        .key(&key)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch object from S3: {}", e),
            )
        })?;

    let file_path = format!("/tmp/{}", &key);

    // Check if the file already exists
    if std::path::Path::new(&file_path).exists() {
        return Ok(StatusCode::OK);
    }

    write_to_file(response.body, &file_path)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to write to file: {}", e),
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
    Ok(())
}

fn get_object_key(url: &str) -> Result<String, url::ParseError> {
    let parsed = Url::parse(url)?;
    let cleaned = &parsed[..Position::AfterPath];
    Ok(cleaned.split('/').last().unwrap_or("").to_string())
}
