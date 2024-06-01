use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/api/v1/tracing", post(tracing))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Span {
    id: String,
    parent_span_id: Option<String>,
    start_time: u64,
    end_time: u64,
    operation_name: String,
    child_spans: Vec<Span>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TracingPayload {
    traces: Vec<Span>
}

async fn tracing(Json(payload): Json<TracingPayload>) -> (StatusCode, Json<()>) {
    println!("{:?}", payload);
    (StatusCode::OK, Json(()))
}
