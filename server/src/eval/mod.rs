use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use serde_json::json;

use diesel::prelude::*;

use ellmo_db::models::{eval::Eval, eval_result::EvalResult, repository::DieselRepository};

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_all))
        .route("/:eval_id/results", get(get_eval_results))
}

async fn get_all() -> impl IntoResponse {
    let mut conn = ellmo_db::establish_connection();
    let repo = DieselRepository::new(&mut conn, ellmo_db::schema::eval::table);

    let evals = repo
        .table
        .order_by(ellmo_db::schema::eval::created_at.desc())
        .load::<Eval>(repo.connection)
        .expect("Failed to fetch eval results");

    (StatusCode::OK, Json(json!({ "evals": evals })))
}

async fn get_eval_results(Path(eval_id): Path<i32>) -> impl IntoResponse {
    let mut conn = ellmo_db::establish_connection();
    let repo = DieselRepository::new(&mut conn, ellmo_db::schema::eval_result::table);

    let result = repo
        .table
        .filter(ellmo_db::schema::eval_result::eval_id.eq(eval_id))
        .get_result::<EvalResult>(repo.connection)
        .expect("Failed to fetch eval result");

    (StatusCode::OK, Json(json!({ "eval_result": result })))
}
