CREATE TABLE span_contexts (
    id SERIAL PRIMARY KEY,
    trace_id INTEGER NOT NULL,
    span_id INTEGER NOT NULL,
    trace_options BYTEA,
    trace_state JSONB
)
