mod eval;

use std::future::Future;
use std::pin::Pin;

use tonic::transport;

use ellmo_proto::ellmo::ellmo_service_server::{EllmoService, EllmoServiceServer};
use ellmo_proto::ellmo::{
    RecordEvalRequest, RecordEvalResponse, ReportSpanRequest, TestExecutionRequest,
};

#[derive(Default)]
struct EllmoRpcDefinition {}

#[tonic::async_trait]
impl EllmoService for EllmoRpcDefinition {
    async fn report_span(
        &self,
        _request: tonic::Request<ReportSpanRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        println!("Received spans!");
        Ok(tonic::Response::new(()))
    }

    async fn queue_test(
        &self,
        request: tonic::Request<TestExecutionRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        println!("Received test execution request!");

        let message = request.into_inner();
        // Each element in the Vec is an encoded argument
        let input_bytes: Vec<Vec<u8>> = message.test_input;
        for bytes in input_bytes {
            let json_str = std::str::from_utf8(&bytes);
            if let Ok(json) = json_str {
                let raw_json: Result<Box<serde_json::value::RawValue>, serde_json::Error> =
                    serde_json::from_str(json);

                println!("{:?}", raw_json.unwrap());
            }
        }

        Ok(tonic::Response::new(()))
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
    pub async fn new(addr: core::net::SocketAddr) -> Self {
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
