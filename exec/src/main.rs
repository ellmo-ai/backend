use olly_db::{
    models::repository::DieselRepository, models::test_registration::TestRegistration,
    schema::test_registration,
};

use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use aws_sdk_s3 as s3;
use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use diesel::prelude::*;
use serde::Deserialize;
use tower_http::cors::CorsLayer;
use url::{ParseError, Position, Url};

use aws_smithy_types::byte_stream::ByteStream;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/execute", post(execute))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
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
    pub test: Test,
    pub input: String,
}

async fn execute(Json(payload): Json<ExecuteRequestPayload>) -> (StatusCode, Json<()>) {
    println!("Received request: {:?}", payload);

    let mut conn = olly_db::establish_connection();

    let repo = DieselRepository {
        connection: &mut conn,
        table: test_registration::table,
    };

    let latest_test_registration = repo
        .table
        .order_by(test_registration::created_at.desc())
        .first::<TestRegistration>(repo.connection)
        .optional();

    let test_registeration = match latest_test_registration {
        Ok(reg) => match reg {
            Some(r) => r,
            None => {
                let error_message = "No test registration found";
                println!("{}", error_message);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(()));
            }
        },
        Err(e) => {
            let error_message = format!("Failed to fetch latest test registration: {}", e);
            println!("{}", error_message);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(()));
        }
    };

    let blob_url = test_registeration.blob_url;
    let key = get_object_key(&blob_url);

    if key.is_err() {
        let error_message = format!("Failed to parse object key: {}", key.err().unwrap());
        println!("{}", error_message);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(()));
    }
    let key = key.unwrap();

    #[allow(deprecated)]
    let config = aws_config::load_from_env().await;
    let client = s3::Client::new(&config);

    let request = client
        .get_object()
        .bucket("test-registrations")
        .key(&key)
        .send()
        .await;

    match request {
        Ok(response) => {
            let stream = response.body;
            let file_path = format!("/tmp/{}", &key);
            let write_result = write_to_file(stream, &file_path).await;

            if write_result.is_err() {
                let error_message =
                    format!("Failed to write to file: {}", write_result.err().unwrap());
                println!("{}", error_message);
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(()));
            }
        }
        Err(e) => {
            let error_message = format!("Failed to fetch object from S3: {}", e);
            println!("{}", error_message);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(()));
        }
    }

    (StatusCode::OK, Json(()))
}

async fn write_to_file(
    mut stream: ByteStream,
    file_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Open the file for writing
    let mut file = File::create(file_path).await?;

    // Stream the ByteStream directly to the file
    while let Some(bytes) = stream.try_next().await? {
        file.write_all(&bytes).await?;
    }

    // Ensure all data is written
    file.flush().await?;

    Ok(())
}

fn get_object_key(url: &str) -> Result<String, ParseError> {
    let parsed = Url::parse(url)?;
    let cleaned: &str = &parsed[..Position::AfterPath];

    let parts: Vec<&str> = cleaned.split('/').collect();
    let key = parts[parts.len() - 1];

    Ok(key.to_string())
}
