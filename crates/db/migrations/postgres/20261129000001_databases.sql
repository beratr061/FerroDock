CREATE TABLE databases (
    id uuid PRIMARY KEY,
    environment_id uuid NOT NULL REFERENCES environments (id) ON DELETE CASCADE,
    name text NOT NULL,
    slug text NOT NULL,
    engine text NOT NULL,
    version text NOT NULL,
    database_name text NOT NULL,
    username text NOT NULL,
    password_enc text NOT NULL,
    root_password_enc text,
    external_port integer UNIQUE CHECK (external_port BETWEEN 1 AND 65535),
    status text NOT NULL DEFAULT 'deploying'
        CHECK (status IN ('deploying', 'running', 'stopped', 'failed')),
    error_code text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (environment_id, slug)
);

CREATE INDEX databases_environment_idx ON databases (environment_id);
