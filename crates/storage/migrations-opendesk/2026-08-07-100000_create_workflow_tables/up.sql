-- opendesk.db: workflow definition tables (email-agent port).
-- Replaces the placeholder script_snippet 话术库 feature with the real
-- email-agent 工作流路由 model: template / binding / stage / rule / script.

-- Clean up orphaned placeholder table from removed migration 2026-07-21-181500.
DROP TABLE IF EXISTS script_snippet;
DROP INDEX IF EXISTS idx_script_snippet_stage;
DROP INDEX IF EXISTS idx_script_snippet_source_id;

CREATE TABLE IF NOT EXISTS workflow_template (
    id              TEXT PRIMARY KEY NOT NULL,      -- '_global' | 'tpl_*'
    name            TEXT NOT NULL,
    template_type   TEXT NOT NULL,                  -- 'email' | 'whatsapp'
    canvas_json     TEXT NOT NULL,                  -- whole workflow-stages{_<id>}.json verbatim
    canvas_version  TEXT,
    canvas_updated  TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS workflow_binding (
    account_id  TEXT PRIMARY KEY NOT NULL,          -- 'acc_*' | 'main'
    template_id TEXT NOT NULL REFERENCES workflow_template(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_workflow_binding_template ON workflow_binding(template_id);

CREATE TABLE IF NOT EXISTS workflow_stage (
    template_id       TEXT NOT NULL REFERENCES workflow_template(id) ON DELETE CASCADE,
    id                TEXT NOT NULL,                -- 's1' | 'agent_s1' ...
    name              TEXT NOT NULL,
    note              TEXT,
    ord               INTEGER NOT NULL,
    ai_level          TEXT,                         -- 'loose' | 'medium' | 'strict'
    x                 INTEGER,
    y                 INTEGER,
    scripts_json      TEXT NOT NULL,                -- JSON array of string
    script_conds_json TEXT NOT NULL,                -- JSON array of string
    PRIMARY KEY (template_id, id)
);
CREATE INDEX IF NOT EXISTS idx_workflow_stage_template ON workflow_stage(template_id, ord);

CREATE TABLE IF NOT EXISTS workflow_rule (
    id                    TEXT PRIMARY KEY NOT NULL,   -- 'default_inquiry' ...
    name                  TEXT NOT NULL,
    from_stages_json      TEXT NOT NULL,               -- JSON array of string
    to_stage              TEXT NOT NULL,
    trigger_keywords_json TEXT NOT NULL,               -- JSON array of string
    trigger_tags_json     TEXT NOT NULL,               -- JSON array of string
    auto_reply            INTEGER NOT NULL DEFAULT 0,
    auto_advance          INTEGER NOT NULL DEFAULT 0,
    reply_script_id       TEXT
);

CREATE TABLE IF NOT EXISTS workflow_script (
    id               TEXT PRIMARY KEY NOT NULL,     -- 's001'..'s053'
    stage            TEXT,
    category_l1      TEXT,
    category_l2      TEXT,
    trigger_text     TEXT,
    description      TEXT,
    from_stage       TEXT,
    to_stage         TEXT,
    tags_json        TEXT NOT NULL,                 -- JSON array of string
    content          TEXT NOT NULL,
    needs_boss_input INTEGER NOT NULL DEFAULT 0,
    boss_input_hint  TEXT,
    sort_order       INTEGER NOT NULL DEFAULT 0,
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_workflow_script_cat ON workflow_script(category_l1, category_l2, sort_order ASC);
