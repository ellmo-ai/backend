CREATE TABLE spans (
    id SERIAL PRIMARY KEY,
    ts_start TIMESTAMPTZ NOT NULL,
    ts_end TIMESTAMPTZ NOT NULL,
    operation_name TEXT NOT NULL,
    parent_span_id INT,
    external_uuid UUID,
    FOREIGN KEY (parent_span_id) REFERENCES spans (id)
);

CREATE TABLE logs (
    id SERIAL PRIMARY KEY,
    ts TIMESTAMPTZ NOT NULL,
    message TEXT NOT NULL,
    span_id INT NOT NULL,
    FOREIGN KEY (span_id) REFERENCES spans (id)
);
