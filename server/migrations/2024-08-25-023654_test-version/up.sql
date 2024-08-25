CREATE TABLE test_registration (
    id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    blob_url TEXT NOT NULL,
    metadata jsonb DEFAULT '{}'
);

CREATE TABLE test_version (
    id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    test_registration_id INT NOT NULL,
    FOREIGN KEY (test_registration_id) REFERENCES test_registration (id)
);

