mod eval;

use std::future::Future;
use std::pin::Pin;

use ellmo_proto::ellmo::ellmo_service_server::{EllmoService, EllmoServiceServer};
use ellmo_proto::ellmo::{
    RecordEvalRequest, RecordEvalResponse, ReportSpanRequest, TestExecutionRequest,
};
use serde_json::{json, value::RawValue};
use std::net::SocketAddr;
use tonic::{transport, Request, Response, Status};

#[derive(Default)]
struct EllmoRpcDefinition {}

#[tonic::async_trait]
impl EllmoService for EllmoRpcDefinition {
    async fn report_span(
        &self,
        _request: Request<ReportSpanRequest>,
    ) -> Result<Response<()>, Status> {
        println!("Received spans!");
        Ok(Response::new(()))
    }

    async fn queue_test(
        &self,
        request: Request<TestExecutionRequest>,
    ) -> Result<Response<()>, Status> {
        let message = request.into_inner();

        let input = message
            .test_input
            .iter()
            .find_map(|bytes| {
                std::str::from_utf8(bytes)
                    .ok()
                    .and_then(|json_str| serde_json::from_str::<Box<RawValue>>(json_str).ok())
            })
            .ok_or_else(|| Status::invalid_argument("Invalid input"))?;

        let test = message
            .versioned_test
            .ok_or_else(|| Status::invalid_argument("Missing versioned test"))?;

        let payload = json!({
            "test": {
                "name": test.name,
                "version": test.version
            },
            "input": input, // serde_json::to_string(&input).map_err(|_| Status::internal("Failed to serialize input"))?
        });

        println!(
            "Forwarding test execution request to test runner, {} {}",
            test.name, test.version
        );

        let client = reqwest::Client::new();
        let res = client
            .post("http://0.0.0.0:3001/execute")
            .json(&payload)
            .send()
            .await;

        return match res {
            Ok(res) => {
                if res.status().is_success() {
                    Ok(Response::new(()))
                } else {
                    Err(Status::internal("Failed to execute test"))
                }
            }
            Err(_e) => Err(Status::internal("Failed to execute test")),
        };
    }

    async fn record_eval(
        &self,
        request: tonic::Request<RecordEvalRequest>,
    ) -> Result<tonic::Response<RecordEvalResponse>, tonic::Status> {
        eval::record_eval(request).await
    }
}

pub struct RpcServer {
    server: Pin<Box<dyn Future<Output = Result<(), transport::Error>> + Send>>,
}

impl RpcServer {
    pub fn new(addr: core::net::SocketAddr) -> Self {
        let ellmo: EllmoRpcDefinition = EllmoRpcDefinition::default();
        let server = transport::Server::builder()
            .add_service(EllmoServiceServer::new(ellmo))
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
