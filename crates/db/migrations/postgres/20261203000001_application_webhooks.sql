CREATE TABLE application_webhooks (
    application_id uuid PRIMARY KEY REFERENCES applications (id) ON DELETE CASCADE,
    secret_enc text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    last_delivery_at timestamptz,
    last_outcome text
);
