CREATE TABLE backup_destinations (
    id uuid PRIMARY KEY,
    organization_id uuid NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    name text NOT NULL,
    kind text NOT NULL CHECK (kind IN ('local', 's3')),
    endpoint text,
    region text,
    bucket text,
    path_prefix text NOT NULL DEFAULT '',
    access_key_id text,
    secret_access_key_enc text,
    path_style boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (organization_id, name),
    CHECK (
        kind = 'local'
        OR (bucket IS NOT NULL AND region IS NOT NULL
            AND access_key_id IS NOT NULL AND secret_access_key_enc IS NOT NULL)
    )
);

CREATE TABLE backup_policies (
    database_id uuid PRIMARY KEY REFERENCES databases (id) ON DELETE CASCADE,
    destination_id uuid NOT NULL REFERENCES backup_destinations (id) ON DELETE RESTRICT,
    schedule text NOT NULL,
    retention integer NOT NULL CHECK (retention BETWEEN 1 AND 365),
    enabled boolean NOT NULL DEFAULT true,
    next_run_at timestamptz NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX backup_policies_due_idx ON backup_policies (next_run_at) WHERE enabled;

CREATE TABLE backup_runs (
    id uuid PRIMARY KEY,
    database_id uuid NOT NULL REFERENCES databases (id) ON DELETE CASCADE,
    destination_id uuid REFERENCES backup_destinations (id) ON DELETE SET NULL,
    operation text NOT NULL CHECK (operation IN ('backup', 'restore')),
    trigger text NOT NULL CHECK (trigger IN ('schedule', 'manual')),
    source_run_id uuid REFERENCES backup_runs (id) ON DELETE SET NULL,
    status text NOT NULL DEFAULT 'queued'
        CHECK (status IN ('queued', 'running', 'succeeded', 'failed')),
    object_key text,
    size_bytes bigint,
    error_code text,
    created_by uuid,
    created_at timestamptz NOT NULL DEFAULT now(),
    finished_at timestamptz
);

CREATE INDEX backup_runs_database_idx ON backup_runs (database_id, created_at DESC);
