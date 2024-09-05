#![allow(dead_code, unused)]

use std::future::Future;
use std::pin::Pin;

use tonic::transport;

use crate::ollyllm::ollyllm_service_server::{OllyllmService, OllyllmServiceServer};
use crate::ollyllm::{
    EvalResult, RecordEvalRequest, RecordEvalResponse, ReportSpanRequest, TestExecutionRequest,
};

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
    async fn report_span(
        &self,
        _request: tonic::Request<ReportSpanRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        println!("Received!");
        Ok(tonic::Response::new(()))
    }
    async fn record_eval(
        &self,
        _request: tonic::Request<RecordEvalRequest>,
    ) -> Result<tonic::Response<RecordEvalResponse>, tonic::Status> {
        println!("Received!");
        Ok(tonic::Response::new(RecordEvalResponse {
            result: EvalResult::Unknown.into(),
            previous_eval_scores: [].to_vec(),
            message: "".to_string(),
        }))
    }
}

pub struct DummyRpcServer {
    server: Pin<Box<dyn Future<Output = Result<(), transport::Error>> + Send>>,
}

impl DummyRpcServer {
    pub async fn new(addr: core::net::SocketAddr) -> Self {
        let ollyllm: OllyllmRpcDefinition = OllyllmRpcDefinition::default();
        let server = transport::Server::builder()
            .add_service(OllyllmServiceServer::new(ollyllm))
            .serve(addr);

        DummyRpcServer {
            server: Box::pin(server),
        }
    }

    pub async fn serve(self) -> Result<(), transport::Error> {
        self.server.await?;
        Ok(())
    }
}
