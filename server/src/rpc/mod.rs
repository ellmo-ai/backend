use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use tonic::transport;

use polay_proto::polay::polay_service_server::{PolayService, PolayServiceServer};
use polay_proto::polay::{
    EvalOutcome, MeaningfulEvalScore, RecordEvalRequest, RecordEvalResponse, ReportSpanRequest,
    TestExecutionRequest,
};

use diesel::prelude::*;
use polay_db::models::{
    eval_result::{EvalResult, EvalRunScores, InsertableEvalResult, SingleEvalScore},
    eval_version::{EvalVersion, InsertableEvalVersion},
    repository::{DieselRepository, Repository},
};
use polay_db::schema::{eval_result, eval_version};

#[derive(Default)]
struct PolayRpcDefinition {}

#[tonic::async_trait]
impl PolayService for PolayRpcDefinition {
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

        let mut conn = polay_db::establish_connection();

        let mut repo = DieselRepository {
            connection: &mut conn,
            table: polay_db::schema::eval_version::table,
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
            table: polay_db::schema::eval_result::table,
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

            let (result, meaningful_scores) = compare_results(previous_results, scores);

            Ok(tonic::Response::new(RecordEvalResponse {
                outcome: result.into(),
                previous_eval_scores: [].to_vec(),
                meaningful_eval_scores: meaningful_scores,
                message: "Success".to_string(),
            }))
        } else {
            Ok(tonic::Response::new(RecordEvalResponse {
                outcome: EvalOutcome::NoChange.into(),
                previous_eval_scores: [].to_vec(),
                meaningful_eval_scores: [].to_vec(),
                message: "Success".to_string(),
            }))
        }
    }
}

fn compare_results(
    previous: EvalRunScores,
    current: EvalRunScores,
) -> (EvalOutcome, Vec<MeaningfulEvalScore>) {
    const INDIVIDUAL_THRESHOLD: f32 = 0.10; // 10% change
    const MEAN_THRESHOLD: f32 = 0.01; // 1% change
    const CONSISTENCY_THRESHOLD: f32 = 0.7;

    let mut grouped_scores: HashMap<String, Vec<(f32, bool)>> = HashMap::new();
    for score in previous.into_iter() {
        grouped_scores
            .entry(score.eval_hash.clone())
            .or_default()
            .push((score.score, false));
    }
    for score in current.into_iter() {
        grouped_scores
            .entry(score.eval_hash.clone())
            .or_default()
            .push((score.score, true));
    }

    let mut percent_changes: Vec<f32> = Vec::new();
    let mut meaningful_changes: Vec<MeaningfulEvalScore> = Vec::new();

    for (eval_hash, scores) in grouped_scores.iter() {
        if scores.len() == 2 {
            let previous_score = scores[0].0;
            let current_score = scores[1].0;

            // Calculate percentage change
            let percent_change = if previous_score != 0.0 {
                (current_score - previous_score) / previous_score.abs()
            } else if current_score != 0.0 {
                1.0 // If previous was 0 and current is not, consider it a 100% increase
            } else {
                0.0 // Both scores are 0, no change
            };

            percent_changes.push(percent_change);

            let individual_outcome = if percent_change > INDIVIDUAL_THRESHOLD {
                EvalOutcome::Improvement
            } else if percent_change < -INDIVIDUAL_THRESHOLD {
                EvalOutcome::Regression
            } else {
                EvalOutcome::NoChange
            };

            if individual_outcome != EvalOutcome::NoChange {
                meaningful_changes.push(MeaningfulEvalScore {
                    eval_hash: eval_hash.clone(),
                    previous_score,
                    current_score,
                    outcome: individual_outcome.into(),
                });
            }
        }
    }

    if percent_changes.is_empty() {
        return (EvalOutcome::Unknown, meaningful_changes);
    }

    let total_percent_change: f32 = percent_changes.iter().sum();
    let mean_percent_change = total_percent_change / percent_changes.len() as f32;
    let num_positive = percent_changes.iter().filter(|&&c| c > 0.0).count();
    let num_negative = percent_changes.iter().filter(|&&c| c < 0.0).count();

    let significant_positives = percent_changes
        .iter()
        .filter(|&&c| c > INDIVIDUAL_THRESHOLD)
        .count();
    let significant_negatives = percent_changes
        .iter()
        .filter(|&&c| c < -INDIVIDUAL_THRESHOLD)
        .count();

    let overall_outcome = if significant_positives > 0 || significant_negatives > 0 {
        match significant_positives.cmp(&significant_negatives) {
            std::cmp::Ordering::Greater => EvalOutcome::Improvement,
            std::cmp::Ordering::Less => EvalOutcome::Regression,
            std::cmp::Ordering::Equal => EvalOutcome::Unknown,
        }
    } else if mean_percent_change.abs() > MEAN_THRESHOLD {
        let total = percent_changes.len() as f32;
        if (num_positive as f32 / total) > CONSISTENCY_THRESHOLD {
            EvalOutcome::Improvement
        } else if (num_negative as f32 / total) > CONSISTENCY_THRESHOLD {
            EvalOutcome::Regression
        } else {
            EvalOutcome::Unknown
        }
    } else {
        EvalOutcome::NoChange
    };

    (overall_outcome, meaningful_changes)
}

pub struct RpcServer {
    server: Pin<Box<dyn Future<Output = Result<(), transport::Error>> + Send>>,
}

impl RpcServer {
    pub async fn new(addr: core::net::SocketAddr) -> Self {
        let polay: PolayRpcDefinition = PolayRpcDefinition::default();
        let server = transport::Server::builder()
            .add_service(PolayServiceServer::new(polay))
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
