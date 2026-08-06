-- Allow the built-in chat LLM to retrieve from the knowledge base on chat send.
-- SQLite / Diesel 不会根据 schema.rs 自动加列，必须显式迁移。

ALTER TABLE llm_setting ADD COLUMN knowledge_enabled INTEGER NOT NULL DEFAULT 1;
