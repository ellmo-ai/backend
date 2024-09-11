#![allow(dead_code, unused)]

use std::future::Future;
use std::pin::Pin;

use tonic::transport;

use crate::ellmo::ellmo_service_server::{EllmoService, EllmoServiceServer};
use crate::ellmo::{
    EvalOutcome, RecordEvalRequest, RecordEvalResponse, ReportSpanRequest, TestExecutionRequest,
};

#[derive(Default)]
struct EllmoRpcDefinition {}

#[tonic::async_trait]
impl EllmoService for EllmoRpcDefinition {
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
            outcome: EvalOutcome::Unknown.into(),
            previous_eval_scores: [].to_vec(),
            meaningful_eval_scores: [].to_vec(),
            message: "".to_string(),
        }))
    }
}

pub struct DummyRpcServer {
    server: Pin<Box<dyn Future<Output = Result<(), transport::Error>> + Send>>,
}

impl DummyRpcServer {
    pub async fn new(addr: core::net::SocketAddr) -> Self {
        let ellmo: EllmoRpcDefinition = EllmoRpcDefinition::default();
        let server = transport::Server::builder()
            .add_service(EllmoServiceServer::new(ellmo))
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
