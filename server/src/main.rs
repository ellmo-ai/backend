mod queue;
mod register;
mod rpc;
mod tracing;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;

use rpc::RpcServer;

#[tokio::main]
async fn main() {
    tokio::task::spawn(async {
        let app = Router::new()
            .route("/", get(root))
            .route("/api/v1/tracing", post(tracing::post))
            .route("/api/v1/test/register", post(register::test_post))
            .layer(CorsLayer::permissive());

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        println!("HTTP listening on {}", listener.local_addr().unwrap());

        axum::serve(listener, app).await.unwrap();
    });

    let addr: core::net::SocketAddr = "[::1]:50051".parse().unwrap();
    let server: RpcServer = RpcServer::new(addr);
    println!("gRPC listening on {}", addr);
    server.serve().await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}
