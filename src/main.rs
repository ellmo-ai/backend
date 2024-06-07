use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::TimeZone;
use ollyllm::db;
use ollyllm::db::models::repository::{DieselRepository, Repository};
use serde::Deserialize;
use tower_http::cors::CorsLayer;

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
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    parent_span_id: Option<String>,
    start_time: u64,
    end_time: u64,
    operation_name: String,
    child_spans: Vec<Span>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TracingPayload {
    traces: Vec<Span>,
}

async fn tracing(Json(payload): Json<TracingPayload>) -> (StatusCode, Json<()>) {
    let traces = payload.traces;

    let mut conn = db::establish_connection();

    let mut repo = DieselRepository {
        connection: &mut conn,
        table: db::schema::spans::table,
    };

    fn process_span(span: Span, repo: &mut DieselRepository<db::schema::spans::table>) {
        // Convert start and end times to chrono::DateTime
        let start_time = chrono::Utc.timestamp_millis_opt(span.start_time as i64);
        let end_time = chrono::Utc.timestamp_millis_opt(span.end_time as i64);

        // Check if both start and end times are valid
        if let (
            chrono::LocalResult::Single(valid_start_time),
            chrono::LocalResult::Single(valid_end_time),
        ) = (start_time, end_time)
        {
            // Create a new InsertableSpan
            let insertable_span = db::models::span::InsertableSpan {
                ts_start: valid_start_time,
                ts_end: valid_end_time,
                operation_name: span.operation_name,
                attribute_ids: vec![],
                event_ids: vec![],
                link_ids: vec![],
            };

            // Attempt to create the span in the repository
            match repo.create(&insertable_span) {
                Ok(_) => println!("Span created"),
                Err(e) => println!("Error creating span: {:?}", e),
            }
        } else {
            // If start or end time is invalid, process child spans
            println!("Invalid start/end time");
            for child_span in span.child_spans {
                process_span(child_span, repo);
            }
        }
    }

    for span in traces {
        process_span(span, &mut repo);
    }

    (StatusCode::OK, Json(()))
}
