#![allow(dead_code, unused)]

use prost_types::Timestamp;
use std::net::SocketAddr;
use tonic::transport::Channel;

use crate::ollyllm::ollyllm_service_client::OllyllmServiceClient;
use crate::ollyllm::{ReportSpanRequest, Span, TestExecutionRequest, VersionedTest};

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
