CREATE TABLE spans (
    id SERIAL PRIMARY KEY,
    ts_start TIMESTAMPTZ NOT NULL,
    ts_end TIMESTAMPTZ NOT NULL,
    operation_name TEXT NOT NULL,
    attribute_ids INTEGER ARRAY NOT NULL,
    event_ids INTEGER ARRAY NOT NULL,
    link_ids INTEGER ARRAY NOT NULL
)
