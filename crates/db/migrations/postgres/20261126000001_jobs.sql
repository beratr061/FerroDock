CREATE TABLE jobs (
    id uuid PRIMARY KEY,
    organization_id uuid REFERENCES organizations (id) ON DELETE CASCADE,
    kind text NOT NULL,
    queue text NOT NULL,
    group_key text,
    payload jsonb NOT NULL DEFAULT '{}',
    status text NOT NULL CHECK (status IN ('queued', 'running', 'succeeded', 'failed', 'cancelled')),
    attempts integer NOT NULL DEFAULT 0,
    cancel_requested boolean NOT NULL DEFAULT false,
    error_code text,
    locked_by text,
    created_at timestamptz NOT NULL DEFAULT now(),
    started_at timestamptz,
    finished_at timestamptz
);

CREATE INDEX jobs_queued_idx ON jobs (queue, created_at) WHERE status = 'queued';
CREATE INDEX jobs_running_idx ON jobs (queue, group_key) WHERE status = 'running';
CREATE INDEX jobs_org_time_idx ON jobs (organization_id, created_at DESC);

CREATE FUNCTION notify_job_queued() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    PERFORM pg_notify('ferrodock_jobs', NEW.queue);
    RETURN NEW;
END;
$$;

CREATE TRIGGER jobs_notify_queued
    AFTER INSERT ON jobs
    FOR EACH ROW EXECUTE FUNCTION notify_job_queued();
