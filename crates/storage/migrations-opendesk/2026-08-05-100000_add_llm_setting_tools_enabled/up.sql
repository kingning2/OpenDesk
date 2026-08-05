-- Allow the built-in chat LLM to call read-only data-query MCP tools.
-- SQLite / Diesel 不会根据 schema.rs 自动加列，必须显式迁移。

ALTER TABLE llm_setting ADD COLUMN tools_enabled INTEGER NOT NULL DEFAULT 1;
