mod dummy_client;
mod dummy_server;

#[cfg(test)]
mod tests {
    use super::*;

    use dummy_client::DummyClient;
    use dummy_server::DummyRpcServer;

    use std::net::{SocketAddr, TcpListener};
    use tokio::{sync::oneshot, task::JoinHandle};

    struct Fresh;
    impl Fresh {
        fn socker_addr() -> SocketAddr {
            let listener: TcpListener = TcpListener::bind("127.0.0.1:0").unwrap();
            listener.local_addr().unwrap()
        }
    }

    #[tokio::test]
    async fn test_queue_span_creation_request() {
        let addr: SocketAddr = Fresh::socker_addr();
        let (tx, rx): (oneshot::Sender<()>, oneshot::Receiver<()>) = oneshot::channel();

        let server_handle: JoinHandle<()> = tokio::spawn(async move {
            let server: DummyRpcServer = DummyRpcServer::new(addr.clone()).await;

            tx.send(()).unwrap();
            server.serve().await.unwrap();
        });

        rx.await.unwrap();

        let mut client: DummyClient = DummyClient::new(addr).await.unwrap();
        let response: Result<tonic::Response<()>, tonic::Status> =
            client.send_dummy_span_creation_request().await;

        assert!(response.is_ok());

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_queue_test_execution_request() {
        let addr: SocketAddr = Fresh::socker_addr();
        let (tx, rx): (oneshot::Sender<()>, oneshot::Receiver<()>) = oneshot::channel();

        let server_handle: JoinHandle<()> = tokio::spawn(async move {
            let server: DummyRpcServer = DummyRpcServer::new(addr.clone()).await;

            tx.send(()).unwrap();
            server.serve().await.unwrap();
        });

        rx.await.unwrap();

        let mut client: DummyClient = DummyClient::new(addr).await.unwrap();
        let response: Result<tonic::Response<()>, tonic::Status> =
            client.send_dummy_test_execution_request().await;

        assert!(response.is_ok());

        server_handle.abort();
    }
}
