CREATE TABLE deployments (
    id uuid PRIMARY KEY,
    application_id uuid NOT NULL REFERENCES applications (id) ON DELETE CASCADE,
    job_id uuid NOT NULL UNIQUE REFERENCES jobs (id) ON DELETE CASCADE,
    kind text NOT NULL CHECK (kind IN ('deploy', 'rollback')),
    rollback_of uuid REFERENCES deployments (id) ON DELETE SET NULL,
    commit_sha text,
    image_ref text,
    image_id text,
    created_by uuid REFERENCES users (id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX deployments_application_idx ON deployments (application_id, created_at DESC);
