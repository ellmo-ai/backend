// @generated automatically by Diesel CLI.

diesel::table! {
    span_contexts (id) {
        id -> Int4,
        trace_id -> Int4,
        span_id -> Int4,
        trace_options -> Nullable<Bytea>,
        trace_state -> Nullable<Jsonb>,
    }
}

diesel::table! {
    spans (id) {
        id -> Int4,
        ts_start -> Timestamptz,
        ts_end -> Timestamptz,
        operation_name -> Text,
        attribute_ids -> Array<Nullable<Int4>>,
        event_ids -> Array<Nullable<Int4>>,
        link_ids -> Array<Nullable<Int4>>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    span_contexts,
    spans,
);
