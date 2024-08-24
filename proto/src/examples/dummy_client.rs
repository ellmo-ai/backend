use std::net::SocketAddr;

use crate::ollyllm::ollyllm_service_client::OllyllmServiceClient;
use crate::ollyllm::{ReportSpanRequest, Span, TestExecutionRequest, VersionedTest};
use prost_types::Timestamp;
use tonic::transport::Channel;

pub struct Client {
    client: OllyllmServiceClient<Channel>,
}

impl Client {
    pub async fn new(socket_addr: SocketAddr) -> Result<Self, tonic::transport::Error> {
        let client = OllyllmServiceClient::connect(format!("http://{}", socket_addr)).await?;
        Ok(Client { client })
    }

    pub async fn send_dummy_span_creation_request(
        &mut self,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let span = Span {
            id: "12345abcd".to_string(),
            start_timestamp: Some(Timestamp {
                seconds: 10,
                nanos: 1,
            }),
            end_timestamp: None,
            operation_name: "start call to openai".to_string(),
            parent_id: "parent_of_12345abcd".to_string(),
            trace_id: "trace_uuid".to_string(),
        };

        let span_request: tonic::Request<ReportSpanRequest> =
            tonic::Request::new(ReportSpanRequest { spans: vec![span] });
        self.client.report_span(span_request).await
    }

    pub async fn send_dummy_test_execution_request(
        &mut self,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let test_request: tonic::Request<TestExecutionRequest> =
            tonic::Request::new(TestExecutionRequest {
                span_id: "12345abcd".to_string(),
                versioned_test: Some(VersionedTest {
                    name: "no_capitals".to_string(),
                    version: 1,
                }),
                request_timestamp: Some(Timestamp {
                    seconds: 100,
                    nanos: 10,
                }),
                test_input: Vec::new(),
            });
        self.client.queue_test(test_request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::RpcServer;
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
            let server: RpcServer = RpcServer::new(addr.clone()).await;

            tx.send(()).unwrap();
            server.serve().await.unwrap();
        });

        rx.await.unwrap();

        let mut client: Client = Client::new(addr).await.unwrap();
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
            let server: RpcServer = RpcServer::new(addr.clone()).await;

            tx.send(()).unwrap();
            server.serve().await.unwrap();
        });

        rx.await.unwrap();

        let mut client: Client = Client::new(addr).await.unwrap();
        let response: Result<tonic::Response<()>, tonic::Status> =
            client.send_dummy_test_execution_request().await;

        assert!(response.is_ok());

        server_handle.abort();
    }
}
