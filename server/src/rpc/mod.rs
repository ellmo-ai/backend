use std::future::Future;
use std::pin::Pin;

use tonic::transport;

use olly_proto::ollyllm::ollyllm_service_server::{OllyllmService, OllyllmServiceServer};
use olly_proto::ollyllm::{
    EvalResult, RecordEvalRequest, RecordEvalResponse, ReportSpanRequest, TestExecutionRequest,
};

#[derive(Default)]
struct OllyllmRpcDefinition {}

#[tonic::async_trait]
impl OllyllmService for OllyllmRpcDefinition {
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
        println!("Received!");
        let message = request.into_inner();
        let _versioned_eval = message.versioned_eval.unwrap();
        let _eval_results = message.eval_results;

        Ok(tonic::Response::new(RecordEvalResponse {
            result: EvalResult::Unknown.into(),
            previous_eval_results: [].to_vec(),
        }))
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
