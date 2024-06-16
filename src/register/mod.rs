use crate::exec::ast::parse_js_to_ast;
use crate::exec::js::execute_ast;
use crate::queue::{Job, JOB_QUEUE};

use axum::response::IntoResponse;
use axum::{http::StatusCode, Json};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RegisterTestPayload {
    content: String,
}

pub struct RegisterTestJob {
    content: String,
}

#[async_trait::async_trait]
impl Job for RegisterTestJob {
    async fn execute(&self) {
        let ast = parse_js_to_ast(&self.content);
        let res = execute_ast(ast).await;
        // match res {
        //     Ok(_) => println!("Execution succeeded"),
        //     Err(e) => eprintln!("Execution failed: {}", e),
        // }
    }
}

pub async fn test_post((Json(payload),): (Json<RegisterTestPayload>,)) -> impl IntoResponse {
    let content = payload.content;

    let job = RegisterTestJob { content };
    JOB_QUEUE.lock().unwrap().add_job(Box::new(job));

    (StatusCode::OK, Json(()))
}

pub async fn test_result_post() -> (StatusCode, Json<()>) {
    (StatusCode::OK, Json(()))
}
