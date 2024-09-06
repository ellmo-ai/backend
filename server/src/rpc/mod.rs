use std::future::Future;
use std::pin::Pin;

use tonic::transport;

use olly_proto::ollyllm::ollyllm_service_server::{OllyllmService, OllyllmServiceServer};
use olly_proto::ollyllm::{
    EvalOutcome, RecordEvalRequest, RecordEvalResponse, ReportSpanRequest, TestExecutionRequest,
};

use diesel::prelude::*;
use olly_db::models::{
    eval_result::{EvalResult, EvalRunScores, InsertableEvalResult, SingleEvalScore},
    eval_version::{EvalVersion, InsertableEvalVersion},
    repository::{DieselRepository, Repository},
};
use olly_db::schema::{eval_result, eval_version};

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
        let versioned_eval = message.versioned_eval.unwrap();
        let eval_scores = message.eval_scores;

        let mut conn = olly_db::establish_connection();

        let mut repo = DieselRepository {
            connection: &mut conn,
            table: olly_db::schema::eval_version::table,
        };

        let existing_eval_version = repo
            .table
            .order_by(eval_version::created_at.desc())
            .first::<EvalVersion>(repo.connection)
            .optional()
            .map_err(|e| {
                println!("{}", e);
                tonic::Status::internal("Failed to fetch eval version")
            })?;

        let existing_eval_version = match existing_eval_version {
            Some(v) => v,
            None => {
                // Create a new eval version
                let new_eval_version = InsertableEvalVersion {
                    name: versioned_eval.name,
                    version: versioned_eval.version,
                    created_at: chrono::Utc::now(),
                };

                repo.create(&new_eval_version)
                    .map_err(|_| tonic::Status::internal("Failed to create new eval version"))?
            }
        };

        let mut repo = DieselRepository {
            connection: &mut conn,
            table: olly_db::schema::eval_result::table,
        };

        let previous_eval_result = repo
            .table
            .order_by(eval_result::created_at.desc())
            .first::<EvalResult>(repo.connection)
            .optional()
            .map_err(|_| tonic::Status::internal("Failed to fetch latest test registration"))?;

        let scores: EvalRunScores = eval_scores
            .into_iter()
            .map(|score| SingleEvalScore {
                eval_hash: score.eval_hash.clone(),
                score: score.score,
            })
            .collect();

        let _ = repo
            .create(&InsertableEvalResult {
                eval_version_id: existing_eval_version.id,
                scores: serde_json::to_value(&scores).unwrap(),
                created_at: chrono::Utc::now(),
            })
            .map_err(|_| tonic::Status::internal("Failed to create new eval result"))?;

        if let Some(previous_result) = previous_eval_result {
            let previous_results: EvalRunScores =
                serde_json::from_value(previous_result.scores).unwrap();

            compare_results(previous_results, scores);
        }

        Ok(tonic::Response::new(RecordEvalResponse {
            outcome: EvalOutcome::Unknown.into(),
            previous_eval_scores: [].to_vec(),
            message: "Success".to_string(),
        }))
    }
}

fn compare_results(_previous: EvalRunScores, _current: EvalRunScores) -> EvalOutcome {
    EvalOutcome::Unknown
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
