-- opendesk.db: workflow script snippet import table (email-agent port).

CREATE TABLE IF NOT EXISTS script_snippet (
    id                  TEXT PRIMARY KEY NOT NULL,
    source_id           TEXT NOT NULL,
    title               TEXT NOT NULL,
    stage               TEXT,
    trigger_text        TEXT,
    description         TEXT,
    from_stage          TEXT,
    to_stage            TEXT,
    tags_json           TEXT NOT NULL,
    body_text           TEXT NOT NULL,
    category_l1         TEXT,
    category_l2         TEXT,
    needs_boss_input    INTEGER NOT NULL DEFAULT 0,
    boss_input_hint     TEXT,
    sort_order          INTEGER NOT NULL DEFAULT 0,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_script_snippet_source_id ON script_snippet(source_id);
CREATE INDEX IF NOT EXISTS idx_script_snippet_stage ON script_snippet(stage, category_l1, category_l2, sort_order ASC);
