-- Add has_api_key flag (secret itself stays in OS keyring).
-- SQLite / Diesel 不会根据 schema.rs 自动加列，必须显式迁移。

ALTER TABLE llm_setting ADD COLUMN has_api_key INTEGER NOT NULL DEFAULT 0;
