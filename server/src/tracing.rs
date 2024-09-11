use axum::{http::StatusCode, Json};
use chrono::TimeZone;
use ellmo_db::models::repository::{DieselRepository, Repository};
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

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
pub struct TracingPayload {
    traces: Vec<Span>,
}

pub async fn post(Json(payload): Json<TracingPayload>) -> (StatusCode, Json<()>) {
    let traces = payload.traces;

    let mut conn = ellmo_db::establish_connection();

    let mut repo = DieselRepository {
        connection: &mut conn,
        table: ellmo_db::schema::span::table,
    };

    let mut uuid_to_span_id: HashMap<String, i32> = HashMap::new();

    fn process_span(
        span: Span,
        repo: &mut DieselRepository<ellmo_db::schema::span::table>,
        uuid_to_span_id: &mut HashMap<String, i32>,
    ) {
        // Convert start and end times to chrono::DateTime
        let start_time = chrono::Utc.timestamp_millis_opt(span.start_time as i64);
        let end_time = chrono::Utc.timestamp_millis_opt(span.end_time as i64);

        // Check if both start and end times are valid
        if let (
            chrono::LocalResult::Single(valid_start_time),
            chrono::LocalResult::Single(valid_end_time),
        ) = (start_time, end_time)
        {
            let parent_span_id = span
                .parent_span_id
                .and_then(|uuid| uuid_to_span_id.get(&uuid).copied());

            // Create a new InsertableSpan
            let insertable_span = ellmo_db::models::span::InsertableSpan {
                ts_start: valid_start_time,
                ts_end: valid_end_time,
                operation_name: span.operation_name,
                parent_span_id,
                external_uuid: Uuid::from_str(&span.id).ok(),
            };

            // Attempt to create the span in the repository
            match repo.create(&insertable_span) {
                Ok(created_span) => {
                    // If span was created successfully, add the UUID to span ID mapping
                    println!("Span created successfully");
                    uuid_to_span_id.insert(span.id, created_span.id);
                }
                Err(e) => println!("Error creating span: {:?}", e),
            }
        } else {
            // If start or end time is invalid, process child spans
            println!("Invalid start/end time");
        }

        for child_span in span.child_spans {
            process_span(child_span, repo, uuid_to_span_id);
        }
    }

    for span in traces {
        process_span(span, &mut repo, &mut uuid_to_span_id);
    }

    (StatusCode::OK, Json(()))
}
