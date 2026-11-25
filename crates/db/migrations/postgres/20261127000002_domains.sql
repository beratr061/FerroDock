CREATE TABLE domains (
    id uuid PRIMARY KEY,
    application_id uuid NOT NULL REFERENCES applications (id) ON DELETE CASCADE,
    host text NOT NULL,
    path_prefix text NOT NULL DEFAULT '/',
    container_port integer NOT NULL CHECK (container_port BETWEEN 1 AND 65535),
    https boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (host, path_prefix)
);

CREATE INDEX domains_application_idx ON domains (application_id);
