CREATE TABLE application_variables (
    application_id uuid NOT NULL REFERENCES applications (id) ON DELETE CASCADE,
    key text NOT NULL,
    value_enc text NOT NULL,
    secret boolean NOT NULL DEFAULT false,
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (application_id, key)
);
