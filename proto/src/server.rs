use std::future::Future;
use std::pin::Pin;

use tonic::transport;

use crate::ollyllm::ollyllm_service_server::{OllyllmService, OllyllmServiceServer};
use crate::ollyllm::{SpanCreationRequest, TestExecutionRequest};

#[derive(Default)]
struct OllyllmRpcDefinition {}

#[tonic::async_trait]
impl OllyllmService for OllyllmRpcDefinition {
    async fn queue_test(
        &self,
        _request: tonic::Request<TestExecutionRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        println!("Received!");
        Ok(tonic::Response::new(()))
    }
    async fn queue_span(
        &self,
        _request: tonic::Request<SpanCreationRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        println!("Received!");
        Ok(tonic::Response::new(()))
    }
}

pub struct RpcServer {
    server: Pin<Box<dyn Future<Output = Result<(), transport::Error>> + Send>>,
}

impl RpcServer {
    pub async fn new(addr: core::net::SocketAddr) -> Self {
        let ollyllm: OllyllmRpcDefinition = OllyllmRpcDefinition::default();
        let server = transport::Server::builder()
            .add_service(OllyllmServiceServer::new(ollyllm))
            .serve(addr);

        RpcServer {
            server: Box::pin(server),
        }
    }

    pub async fn serve(self) -> Result<(), transport::Error> {
        self.server.await?;
        Ok(())
    }
}
