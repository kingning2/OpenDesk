-- opendesk.db: background job queue for Worker coordination.

CREATE TABLE IF NOT EXISTS background_job (
    id              TEXT PRIMARY KEY NOT NULL,
    job_type        TEXT NOT NULL,
    payload_json    TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'queued',
    progress        REAL NOT NULL DEFAULT 0,
    error_message   TEXT,
    worker_pid      INTEGER,
    created_at      TEXT NOT NULL,
    started_at      TEXT,
    completed_at    TEXT
);

CREATE INDEX IF NOT EXISTS idx_background_job_queue
    ON background_job(status, created_at);

CREATE INDEX IF NOT EXISTS idx_background_job_type
    ON background_job(job_type, status);
