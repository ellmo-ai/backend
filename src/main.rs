use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
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

struct TracingPayload {

}

async fn tracing() -> (StatusCode, Json<()>) {
    (StatusCode::OK, Json(()))
}
