use std::collections::HashMap;
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

            let result = compare_results(previous_results, scores);

            Ok(tonic::Response::new(RecordEvalResponse {
                outcome: result.into(),
                previous_eval_scores: [].to_vec(),
                message: "Success".to_string(),
            }))
        } else {
            Ok(tonic::Response::new(RecordEvalResponse {
                outcome: EvalOutcome::Improvement.into(),
                previous_eval_scores: [].to_vec(),
                message: "Success".to_string(),
            }))
        }
    }
}

fn compare_results(previous: EvalRunScores, current: EvalRunScores) -> EvalOutcome {
    const INDIVIDUAL_THRESHOLD: f32 = 0.1;
    const MEAN_THRESHOLD: f32 = 0.01;
    const CONSISTENCY_THRESHOLD: f32 = 0.7;

    // Group scores by eval_hash (same eval input/expected)
    let mut grouped_scores: HashMap<String, Vec<f32>> = HashMap::new();
    for score in previous.into_iter().chain(current.into_iter()) {
        grouped_scores
            .entry(score.eval_hash)
            .or_default()
            .push(score.score);
    }

    // Calculate differences for groups with a before and after score
    let differences: Vec<f32> = grouped_scores
        .values()
        .filter(|scores| scores.len() == 2)
        .map(|scores| scores[1] - scores[0])
        .collect();

    if differences.is_empty() {
        return EvalOutcome::Unknown;
    }

    println!("{:?}", differences);

    // Analyze the differences
    let total_diff: f32 = differences.iter().sum();
    let mean_diff = total_diff / differences.len() as f32;
    let num_positive = differences.iter().filter(|&&d| d > 0.0).count();
    let num_negative = differences.iter().filter(|&&d| d < 0.0).count();

    // Count significant changes
    let significant_positives = differences
        .iter()
        .filter(|&&d| d > INDIVIDUAL_THRESHOLD)
        .count();
    let significant_negatives = differences
        .iter()
        .filter(|&&d| d < -INDIVIDUAL_THRESHOLD)
        .count();

    // Determine if there's a meaningful change
    if significant_positives > 0 || significant_negatives > 0 {
        match significant_positives.cmp(&significant_negatives) {
            std::cmp::Ordering::Greater => EvalOutcome::Improvement,
            std::cmp::Ordering::Less => EvalOutcome::Regression,
            std::cmp::Ordering::Equal => EvalOutcome::Unknown,
        }
    } else if mean_diff.abs() > MEAN_THRESHOLD {
        let total = differences.len() as f32;
        if (num_positive as f32 / total) > CONSISTENCY_THRESHOLD {
            EvalOutcome::Improvement
        } else if (num_negative as f32 / total) > CONSISTENCY_THRESHOLD {
            EvalOutcome::Regression
        } else {
            EvalOutcome::Unknown
        }
    } else {
        EvalOutcome::NoChange
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
