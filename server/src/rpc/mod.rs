mod eval;

use serde_json::json;
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

        let input: Option<Box<serde_json::value::RawValue>> = {
            // Each element in the Vec is an encoded argument
            let input_bytes: Vec<Vec<u8>> = message.test_input;

            // Initialize the result as None
            let mut result = None;

            for bytes in input_bytes {
                // Attempt to decode the bytes into a string
                if let Ok(json_str) = std::str::from_utf8(&bytes) {
                    // Attempt to parse the string into a RawValue
                    let raw_json: Result<Box<serde_json::value::RawValue>, serde_json::Error> =
                        serde_json::from_str(json_str);

                    // If parsing was successful, set result to Some(raw_json)
                    if raw_json.is_ok() {
                        result = Some(raw_json.unwrap());
                        break; // Exit the loop after the first successful parse
                    }
                }
            }

            result
        };

        if input.is_none() {
            return Err(tonic::Status::invalid_argument("Invalid input"));
        }
        let input = input.unwrap();

        let test = message.versioned_test.unwrap();

        let payload = json!({
            "test": {
                "name": test.name,
                "version": test.version
            },
            "input": input
        });

        let client = reqwest::Client::new();

        let res = client
            .post("http://0.0.0.0:3001/execute")
            .json(&payload)
            .send()
            .await;

        match res {
            Ok(data) => println!("Success: {:?}", data),
            Err(e) => eprintln!("Error: {}", e), // Prints the error
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
