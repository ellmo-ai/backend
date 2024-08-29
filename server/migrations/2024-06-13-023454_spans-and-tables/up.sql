CREATE TABLE span (
    id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    ts_start TIMESTAMPTZ NOT NULL,
    ts_end TIMESTAMPTZ NOT NULL,
    operation_name TEXT NOT NULL,
    parent_span_id INT,
    external_uuid UUID,
    FOREIGN KEY (parent_span_id) REFERENCES span (id)
);

CREATE TABLE log (
    id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    ts TIMESTAMPTZ NOT NULL,
    message TEXT NOT NULL,
    span_id INT NOT NULL,
    FOREIGN KEY (span_id) REFERENCES span (id)
);

