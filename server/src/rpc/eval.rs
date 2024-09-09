use std::collections::HashMap;

use polay_proto::polay::{
    EvalOutcome, EvalScore, MeaningfulEvalScore, RecordEvalRequest, RecordEvalResponse,
};

use diesel::prelude::*;
use polay_db::{
    models::{
        eval::{Eval, InsertableEval},
        eval_result::{EvalResult, EvalRunScores, InsertableEvalResult, SingleEvalScore},
        prompt_version::{InsertablePromptVersion, PromptVersion},
        repository::{DieselRepository, Repository},
    },
    schema::{eval, eval_result, prompt_version},
};

pub async fn record_eval(
    request: tonic::Request<RecordEvalRequest>,
) -> Result<tonic::Response<RecordEvalResponse>, tonic::Status> {
    let message = request.into_inner();
    let eval = message.eval.unwrap();
    let prompt = message.prompt.unwrap();
    let eval_scores = message.eval_scores;
    let base_version = message.base_version;

    let mut conn = polay_db::establish_connection();

    let mut repo = DieselRepository {
        connection: &mut conn,
        table: prompt_version::table,
    };

    // Create a new prompt version if it doesn't exist
    let existing_prompt_version = repo
        .table
        .filter(prompt_version::version.eq(&prompt.version))
        .first::<PromptVersion>(repo.connection)
        .optional()
        .map_err(|_| tonic::Status::internal("Failed to fetch prompt version"))?;

    let prompt_version = if let Some(v) = existing_prompt_version {
        println!("Using existing prompt version");
        v
    } else {
        println!("Creating new prompt version");
        let new_prompt_version = InsertablePromptVersion {
            name: prompt.name,
            version: prompt.version.clone(),
            created_at: chrono::Utc::now(),
        };

        repo.create(&new_prompt_version)
            .map_err(|_| tonic::Status::internal("Failed to create new prompt version"))?
    };

    // Get the prompt version to compare against
    let base_prompt_version = if let Some(base_version) = base_version {
        let res = repo
            .table
            .filter(prompt_version::version.eq(&base_version))
            .first::<PromptVersion>(repo.connection)
            .optional()
            .map_err(|_| tonic::Status::internal("Failed to fetch base prompt version"))?;

        match res {
            Some(v) => Some(v),
            None => return Err(tonic::Status::invalid_argument("Base version not found")),
        }
    } else {
        let prompt_version = &prompt.version;

        // Get one version before the current version
        repo.table
            .filter(prompt_version::version.lt(prompt_version))
            .order(prompt_version::version.desc())
            .first::<PromptVersion>(repo.connection)
            .optional()
            .map_err(|_| tonic::Status::internal("Failed to fetch base prompt version"))?
    };

    println!("Using base prompt version: {:?}", base_prompt_version);

    let mut repo = DieselRepository {
        connection: &mut conn,
        table: eval::table,
    };

    // Create a new eval version if it doesn't exist
    let existing_eval_version = repo
        .table
        .inner_join(prompt_version::table)
        .filter(
            eval::name
                .eq(&eval.name)
                .and(prompt_version::version.eq(&prompt.version)),
        )
        .select(eval::all_columns)
        .first::<Eval>(repo.connection)
        .optional()
        .map_err(|_| tonic::Status::internal("Failed to fetch eval version"))?;

    let existing_eval_version = if let Some(v) = existing_eval_version {
        v
    } else {
        let new_eval_version = InsertableEval {
            name: eval.name,
            prompt_version_id: prompt_version.id,
            created_at: chrono::Utc::now(),
        };

        repo.create(&new_eval_version)
            .map_err(|_| tonic::Status::internal("Failed to create new eval version"))?
    };

    let mut repo = DieselRepository {
        connection: &mut conn,
        table: eval_result::table,
    };

    // Get the last eval result for the base prompt version
    // This will be used to compare against the new eval result
    // If it doesn't exist, we'll just return no change
    let previous_eval_result = if let Some(base_prompt_version) = base_prompt_version {
        repo.table
            .inner_join(eval::table)
            .filter(eval::prompt_version_id.eq(base_prompt_version.id))
            .order(eval_result::created_at.desc())
            .select(eval_result::all_columns)
            .first::<EvalResult>(repo.connection)
            .optional()
            .map_err(|_| tonic::Status::internal("Failed to fetch previous eval result"))?
    } else {
        // We don't have a base version to compare against
        None
    };

    println!("Using previous eval result: {:?}", previous_eval_result);

    let scores: EvalRunScores = eval_scores
        .into_iter()
        .map(|score| SingleEvalScore {
            eval_hash: score.eval_hash.clone(),
            score: score.score,
        })
        .collect();

    let _ = repo
        .create(&InsertableEvalResult {
            eval_id: existing_eval_version.id,
            scores: serde_json::to_value(&scores).unwrap(),
            created_at: chrono::Utc::now(),
        })
        .map_err(|_| tonic::Status::internal("Failed to create new eval result"))?;

    if let Some(previous_result) = previous_eval_result {
        let previous_results: EvalRunScores =
            serde_json::from_value(previous_result.scores).unwrap();

        let (result, meaningful_scores) = compare_results(&previous_results, scores);

        Ok(tonic::Response::new(RecordEvalResponse {
            outcome: result.into(),
            previous_eval_scores: previous_results
                .into_iter()
                .map(|res| EvalScore {
                    eval_hash: res.eval_hash,
                    score: res.score,
                })
                .collect(),
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

fn compare_results(
    previous: &EvalRunScores,
    current: EvalRunScores,
) -> (EvalOutcome, Vec<MeaningfulEvalScore>) {
    const INDIVIDUAL_THRESHOLD: f32 = 0.10; // 10% change
    const MEAN_THRESHOLD: f32 = 0.01; // 1% change
    const CONSISTENCY_THRESHOLD: f32 = 0.7;

    let mut grouped_scores: HashMap<String, Vec<(f32, bool)>> = HashMap::new();
    for score in previous.iter() {
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
