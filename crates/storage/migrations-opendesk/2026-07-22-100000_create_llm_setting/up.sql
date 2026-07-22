-- opendesk.db: single-row LLM provider metadata (API key lives in OS keyring).
-- has_api_key 由后续迁移添加，避免已执行过本迁移的旧库缺列。

CREATE TABLE IF NOT EXISTS llm_setting (
    id              TEXT PRIMARY KEY NOT NULL,
    provider        TEXT NOT NULL,
    base_url        TEXT,
    model_id        TEXT NOT NULL,
    api_key_ref     TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);
