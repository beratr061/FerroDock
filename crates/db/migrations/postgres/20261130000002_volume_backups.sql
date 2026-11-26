ALTER TABLE backup_policies DROP CONSTRAINT backup_policies_pkey;
ALTER TABLE backup_policies ALTER COLUMN database_id DROP NOT NULL;
ALTER TABLE backup_policies
    ADD COLUMN application_id uuid REFERENCES applications (id) ON DELETE CASCADE,
    ADD COLUMN volume text,
    ADD CONSTRAINT backup_policies_one_target CHECK (
        (database_id IS NOT NULL AND application_id IS NULL AND volume IS NULL)
        OR (database_id IS NULL AND application_id IS NOT NULL AND volume IS NOT NULL)
    );
CREATE UNIQUE INDEX backup_policies_database_idx ON backup_policies (database_id)
    WHERE database_id IS NOT NULL;
CREATE UNIQUE INDEX backup_policies_volume_idx ON backup_policies (application_id, volume)
    WHERE application_id IS NOT NULL;

ALTER TABLE backup_runs ALTER COLUMN database_id DROP NOT NULL;
ALTER TABLE backup_runs
    ADD COLUMN application_id uuid REFERENCES applications (id) ON DELETE CASCADE,
    ADD COLUMN volume text,
    ADD CONSTRAINT backup_runs_one_target CHECK (
        (database_id IS NOT NULL AND application_id IS NULL AND volume IS NULL)
        OR (database_id IS NULL AND application_id IS NOT NULL AND volume IS NOT NULL)
    );
CREATE INDEX backup_runs_volume_idx ON backup_runs (application_id, volume, created_at DESC)
    WHERE application_id IS NOT NULL;
