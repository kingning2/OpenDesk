DROP INDEX IF EXISTS idx_crawler_channel_email_status;

-- SQLite cannot DROP COLUMN before 3.35; recreate table for rollback.
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
    custom_url
FROM crawler_channel;

DROP TABLE crawler_channel;

CREATE TABLE crawler_channel (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id TEXT NOT NULL,
    keyword TEXT NOT NULL,
    platform TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    title TEXT NOT NULL,
    country TEXT,
    subscriber_count INTEGER,
    email TEXT,
    description TEXT,
    custom_url TEXT,
    UNIQUE(job_id, channel_id)
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
    custom_url
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
    custom_url
FROM crawler_channel_backup;

DROP TABLE crawler_channel_backup;

CREATE INDEX IF NOT EXISTS idx_crawler_channel_job
    ON crawler_channel(job_id, id);
