-- Key-value settings for crawler (e.g. YouTube API key).

CREATE TABLE IF NOT EXISTS crawler_setting (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
