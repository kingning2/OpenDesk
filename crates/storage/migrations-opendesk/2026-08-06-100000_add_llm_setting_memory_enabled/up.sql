-- Allow the built-in chat LLM to use cross-session long-term memory.
-- SQLite / Diesel 不会根据 schema.rs 自动加列，必须显式迁移。

ALTER TABLE llm_setting ADD COLUMN memory_enabled INTEGER NOT NULL DEFAULT 1;
