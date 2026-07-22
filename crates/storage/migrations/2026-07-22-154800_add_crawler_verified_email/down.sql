-- SQLite cannot drop columns before 3.35; rebuild table without verified_email.
CREATE TABLE crawler_channel_backup AS
SELECT
    id,
    job_id,
    keyword,
    platform,
    channel_id,
    title,
    country,
    subscriber_count,
    email,
    description,
    custom_url,
    email_status,
    enrich_attempts,
    enrich_error,
    enriched_at
FROM crawler_channel;

DROP TABLE crawler_channel;

CREATE TABLE crawler_channel (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    job_id TEXT NOT NULL,
    keyword TEXT NOT NULL,
    platform TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    title TEXT NOT NULL,
    country TEXT,
    subscriber_count BIGINT,
    email TEXT,
    description TEXT,
    custom_url TEXT,
    email_status TEXT NOT NULL DEFAULT 'pending_enrich',
    enrich_attempts INTEGER NOT NULL DEFAULT 0,
    enrich_error TEXT,
    enriched_at TEXT
);

INSERT INTO crawler_channel (
    id,
    job_id,
    keyword,
    platform,
    channel_id,
    title,
    country,
    subscriber_count,
    email,
    description,
    custom_url,
    email_status,
    enrich_attempts,
    enrich_error,
    enriched_at
)
SELECT
    id,
    job_id,
    keyword,
    platform,
    channel_id,
    title,
    country,
    subscriber_count,
    email,
    description,
    custom_url,
    email_status,
    enrich_attempts,
    enrich_error,
    enriched_at
FROM crawler_channel_backup;

DROP TABLE crawler_channel_backup;

CREATE INDEX IF NOT EXISTS idx_crawler_channel_job
    ON crawler_channel(job_id, id);

CREATE INDEX IF NOT EXISTS idx_crawler_channel_email_status
    ON crawler_channel(job_id, email_status);
