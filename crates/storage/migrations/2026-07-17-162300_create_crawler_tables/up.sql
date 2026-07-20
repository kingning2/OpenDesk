-- Crawler keyword + channel tables (shared crawler.db).

CREATE TABLE IF NOT EXISTS crawler_keyword (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    batch_id TEXT NOT NULL,
    text TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    UNIQUE(batch_id, text)
);

CREATE INDEX IF NOT EXISTS idx_crawler_keyword_batch
    ON crawler_keyword(batch_id, enabled);

CREATE TABLE IF NOT EXISTS crawler_channel (
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

CREATE INDEX IF NOT EXISTS idx_crawler_channel_job
    ON crawler_channel(job_id, id);
