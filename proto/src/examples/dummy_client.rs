#![allow(dead_code, unused)]

use prost_types::Timestamp;
use std::net::SocketAddr;
use tonic::transport::Channel;

use crate::polay::polay_service_client::PolayServiceClient;
use crate::polay::{ReportSpanRequest, Span, TestExecutionRequest, VersionedTest};

pub struct DummyClient {
    client: PolayServiceClient<Channel>,
}

impl DummyClient {
    pub async fn new(socket_addr: SocketAddr) -> Result<Self, tonic::transport::Error> {
        let client = PolayServiceClient::connect(format!("http://{}", socket_addr)).await?;
        Ok(DummyClient { client })
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
            parent_id: Some("parent_of_12345abcd".to_string()),
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
                span_id: Some("12345abcd".to_string()),
                versioned_test: Some(VersionedTest {
                    name: "no_capitals".to_string(),
                    version: "1.0.0".to_string(),
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
