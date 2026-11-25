CREATE TABLE login_attempts (
    user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    source text NOT NULL,
    failures integer NOT NULL DEFAULT 0,
    window_started_at timestamptz NOT NULL,
    locked_until timestamptz,
    PRIMARY KEY (user_id, source)
);

CREATE INDEX login_attempts_window_idx ON login_attempts (window_started_at);
