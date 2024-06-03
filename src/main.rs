use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::TimeZone;
use serde::Deserialize;
use tower_http::cors::CorsLayer;
use ollyllm::db;
use ollyllm::db::models::repository::Repository;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/api/v1/tracing", post(tracing))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Span {
    id: String,
    parent_span_id: Option<String>,
    start_time: u64,
    end_time: u64,
    operation_name: String,
    child_spans: Vec<Span>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TracingPayload {
    traces: Vec<Span>
}

async fn tracing(Json(payload): Json<TracingPayload>) -> (StatusCode, Json<()>) {
    let traces = payload.traces;

    let mut conn = db::establish_connection();

    let mut repo = db::models::repository::DieselRepository {
        connection: &mut conn,
        table: db::schema::spans::table,
    };

    for span in traces {
        let ts_start = chrono::Utc.timestamp_millis_opt(span.start_time as i64);
        let ts_end = chrono::Utc.timestamp_millis_opt(span.end_time as i64);

        let span = if let (chrono::LocalResult::Single(ts_start), chrono::LocalResult::Single(ts_end)) = (ts_start, ts_end) {
            db::models::span::InsertableSpan {
                ts_start,
                ts_end,
                operation_name: span.operation_name,
                attribute_ids: vec![],
                event_ids: vec![],
                link_ids: vec![],
            }
        } else {
            println!("Invalid start/end time");
            continue;
        };

        let res = repo.create(&span);
        match res {
            Ok(_) => println!("Span created"),
            Err(e) => println!("Error creating span: {:?}", e),
        }
    }

    (StatusCode::OK, Json(()))
}
