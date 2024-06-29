mod db;
mod exec;
mod queue;
mod register;
mod tracing;

use server::register;
use server::tracing;

use axum::{
    routing::{get, post},
    Router,
};

use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/api/v1/tracing", post(tracing::post))
        .route("/register/test", post(register::test_post))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}
