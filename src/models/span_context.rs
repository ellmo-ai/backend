use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::span_contexts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SpanContext {
    pub id: i32,
    pub trace_id: i32,
    pub span_id: i32,
    pub trace_options: Option<Vec<u8>>,
    pub trace_state: Option<serde_json::Value>,
}
