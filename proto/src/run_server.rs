use proto::server::RpcServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr: core::net::SocketAddr = "[::1]:50051".parse().unwrap();

    let server: RpcServer = RpcServer::new(addr).await;
    server.serve().await?;

    Ok(())
}
