CREATE TABLE prompt_version (
    id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE eval (
    id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    prompt_version_id INT NOT NULL,
    FOREIGN KEY (prompt_version_id) REFERENCES prompt_version (id) ON DELETE CASCADE
);

CREATE TABLE eval_result (
    id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    eval_id INT NOT NULL,
    scores jsonb DEFAULT '{}' NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    FOREIGN KEY (eval_id) REFERENCES eval (id) ON DELETE CASCADE
);
