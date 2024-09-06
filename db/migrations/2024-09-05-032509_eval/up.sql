CREATE TABLE eval_version (
    id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE eval_result (
    id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    eval_version_id INT NOT NULL,
    scores jsonb DEFAULT '{}' NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    FOREIGN KEY (eval_version_id) REFERENCES eval_version (id)
);

