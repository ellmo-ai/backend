// @generated automatically by Diesel CLI.

diesel::table! {
    logs (id) {
        id -> Int4,
        ts -> Timestamptz,
        message -> Text,
        span_id -> Int4,
    }
}

diesel::table! {
    spans (id) {
        id -> Int4,
        ts_start -> Timestamptz,
        ts_end -> Timestamptz,
        operation_name -> Text,
        parent_span_id -> Nullable<Int4>,
        external_uuid -> Nullable<Uuid>,
    }
}

diesel::joinable!(logs -> spans (span_id));

diesel::allow_tables_to_appear_in_same_query!(logs, spans,);
