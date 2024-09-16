use chrono::Utc;
use diesel::prelude::*;
use std::collections::HashMap;
use tonic::{Request, Response, Status};

use ellmo_db::{
    establish_connection,
    models::{
        eval::{Eval, InsertableEval},
        eval_result::{EvalResult, EvalRunScores, InsertableEvalResult, SingleEvalScore},
        prompt_version::{InsertablePromptVersion, PromptVersion},
        repository::{DieselRepository, Repository},
    },
    schema::{eval, eval_result, prompt_version},
};
use ellmo_proto::ellmo::{
    EvalOutcome, EvalScore, MeaningfulEvalScore, RecordEvalRequest, RecordEvalResponse,
    VersionedPrompt,
};

/// Record an eval run and compare it to a previous run
pub async fn record_eval(
    request: Request<RecordEvalRequest>,
) -> Result<Response<RecordEvalResponse>, Status> {
    let message = request.into_inner();

    let eval = message
        .eval
        .ok_or_else(|| Status::invalid_argument("Missing eval"))?;
    let prompt = message
        .prompt
        .ok_or_else(|| Status::invalid_argument("Missing prompt"))?;

    let eval_scores = message.eval_scores;
    let base_version = message.base_version;

    let mut conn = establish_connection();

    // Get or create the prompt version being evaluated
    let prompt_version = get_or_create_prompt_version(&mut conn, &prompt)?;
    let existing_eval_version = get_or_create_eval_version(&mut conn, &eval, &prompt_version)?;

    // Get the previous eval result for the base version, if it exists
    let previous_eval_result = get_previous_eval_result(&mut conn, &prompt, base_version)?;

    let scores: EvalRunScores = convert_eval_scores(eval_scores);
    create_new_eval_result(&mut conn, &existing_eval_version, &scores)?;

    if let Some(previous_result) = previous_eval_result {
        let previous_results: EvalRunScores = serde_json::from_value(previous_result.scores)
            .map_err(|_| Status::internal("Failed to deserialize previous scores"))?;

        let (result, meaningful_scores) = compare_results(&previous_results, scores);

        Ok(Response::new(RecordEvalResponse {
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
        Ok(Response::new(RecordEvalResponse {
            outcome: EvalOutcome::NoChange.into(),
            previous_eval_scores: Vec::new(),
            meaningful_eval_scores: Vec::new(),
            message: "Success".to_string(),
        }))
    }
}

fn get_or_create_prompt_version(
    conn: &mut PgConnection,
    prompt: &VersionedPrompt,
) -> Result<PromptVersion, Status> {
    let mut repo = DieselRepository::new(conn, prompt_version::table);

    let existing = repo
        .table
        .filter(prompt_version::version.eq(&prompt.version))
        .first::<PromptVersion>(repo.connection)
        .optional()
        .map_err(|_| Status::internal("Failed to fetch prompt version"))?;

    match existing {
        Some(version) => Ok(version),
        None => {
            let new_prompt_version = InsertablePromptVersion {
                name: prompt.name.clone(),
                version: prompt.version.clone(),
                created_at: Utc::now(),
            };

            repo.create(&new_prompt_version)
                .map_err(|_| Status::internal("Failed to create prompt version"))
        }
    }
}

fn get_or_create_eval_version(
    conn: &mut PgConnection,
    eval: &ellmo_proto::ellmo::Eval,
    prompt_version: &PromptVersion,
) -> Result<Eval, Status> {
    let mut repo = DieselRepository::new(conn, eval::table);

    let existing = repo
        .table
        .inner_join(prompt_version::table)
        .filter(
            eval::name
                .eq(&eval.name)
                .and(prompt_version::version.eq(&prompt_version.version)),
        )
        .select(eval::all_columns)
        .first::<Eval>(repo.connection)
        .optional()
        .map_err(|_| Status::internal("Failed to fetch eval version"))?;

    match existing {
        Some(version) => Ok(version),
        None => {
            let new_eval_version = InsertableEval {
                name: eval.name.clone(),
                prompt_version_id: prompt_version.id,
                created_at: Utc::now(),
            };

            repo.create(&new_eval_version)
                .map_err(|_| Status::internal("Failed to create eval version"))
        }
    }
}

fn get_previous_eval_result(
    conn: &mut PgConnection,
    prompt: &VersionedPrompt,
    base_version: Option<String>,
) -> Result<Option<EvalResult>, Status> {
    let base_prompt_version = match get_base_prompt_version(conn, prompt, base_version)? {
        Some(version) => version,
        None => return Ok(None),
    };

    let repo = DieselRepository::new(conn, eval_result::table);

    eval_result::table
        .inner_join(eval::table)
        .filter(eval::prompt_version_id.eq(base_prompt_version.id))
        .order(eval_result::created_at.desc())
        .select(eval_result::all_columns)
        .first::<EvalResult>(repo.connection)
        .optional()
        .map_err(|_| Status::internal("Failed to fetch previous eval result"))
}

fn get_base_prompt_version(
    conn: &mut PgConnection,
    prompt: &VersionedPrompt,
    base_version: Option<String>,
) -> Result<Option<PromptVersion>, Status> {
    let repo = DieselRepository::new(conn, prompt_version::table);

    if let Some(base_version) = base_version {
        let res = repo
            .table
            .filter(prompt_version::version.eq(&base_version))
            .first::<PromptVersion>(conn)
            .optional()
            .map_err(|_| Status::internal("Failed to fetch base prompt version"))?
            .ok_or_else(|| Status::invalid_argument("Base version not found"))?;

        Ok(Some(res))
    } else {
        let res = repo
            .table
            .filter(prompt_version::version.lt(&prompt.version))
            .order(prompt_version::version.desc())
            .first::<PromptVersion>(conn)
            .optional()
            .map_err(|_| Status::internal("Failed to fetch base prompt version"))?;

        Ok(res)
    }
}

fn convert_eval_scores(eval_scores: Vec<EvalScore>) -> EvalRunScores {
    eval_scores
        .into_iter()
        .map(|score| SingleEvalScore {
            eval_hash: score.eval_hash,
            score: score.score,
        })
        .collect()
}

fn create_new_eval_result(
    conn: &mut PgConnection,
    existing_eval_version: &Eval,
    scores: &EvalRunScores,
) -> Result<EvalResult, Status> {
    let mut repo = DieselRepository::new(conn, eval_result::table);

    let new_eval_result = repo.create(&InsertableEvalResult {
        eval_id: existing_eval_version.id,
        scores: serde_json::to_value(scores)
            .map_err(|_| Status::internal("Failed to serialize scores"))?,
        created_at: Utc::now(),
    });

    new_eval_result.map_err(|_| Status::internal("Failed to create eval result"))
}

fn compare_results(
    previous: &EvalRunScores,
    current: EvalRunScores,
) -> (EvalOutcome, Vec<MeaningfulEvalScore>) {
    const INDIVIDUAL_THRESHOLD: f32 = 0.10;
    const MEAN_THRESHOLD: f32 = 0.01;
    const CONSISTENCY_THRESHOLD: f32 = 0.7;

    let mut grouped_scores: HashMap<String, Vec<(f32, bool)>> = HashMap::new();
    previous.iter().for_each(|score| {
        grouped_scores
            .entry(score.eval_hash.clone())
            .or_default()
            .push((score.score, false));
    });
    current.into_iter().for_each(|score| {
        grouped_scores
            .entry(score.eval_hash.clone())
            .or_default()
            .push((score.score, true));
    });

    let mut percent_changes = Vec::new();
    let mut meaningful_changes = Vec::new();

    for (eval_hash, scores) in grouped_scores.iter() {
        if scores.len() == 2 {
            let previous_score = scores[0].0;
            let current_score = scores[1].0;

            let percent_change = if previous_score != 0.0 {
                (current_score - previous_score) / previous_score.abs()
            } else if current_score != 0.0 {
                1.0
            } else {
                0.0
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
    } else {
        let mean_percent_change =
            percent_changes.iter().sum::<f32>() / percent_changes.len() as f32;
        if mean_percent_change.abs() > MEAN_THRESHOLD {
            let total = percent_changes.len() as f32;
            let num_positive = percent_changes.iter().filter(|&&c| c > 0.0).count() as f32;
            let num_negative = percent_changes.iter().filter(|&&c| c < 0.0).count() as f32;

            if num_positive / total > CONSISTENCY_THRESHOLD {
                EvalOutcome::Improvement
            } else if num_negative / total > CONSISTENCY_THRESHOLD {
                EvalOutcome::Regression
            } else {
                EvalOutcome::Unknown
            }
        } else {
            EvalOutcome::NoChange
        }
    };

    (overall_outcome, meaningful_changes)
}
