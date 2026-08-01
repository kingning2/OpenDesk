-- opendesk.db: workflow runtime checkpoint tables

CREATE TABLE IF NOT EXISTS wf_rt_instance (
    instance_id      TEXT PRIMARY KEY NOT NULL,
    definition_id    TEXT,
    definition_json  TEXT NOT NULL,
    state            TEXT NOT NULL,
    context_json     TEXT NOT NULL,
    context_version  BIGINT NOT NULL DEFAULT 0,
    error_code       TEXT,
    error_message    TEXT,
    heartbeat_at     TEXT,
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL,
    started_at       TEXT,
    finished_at      TEXT
);

CREATE INDEX IF NOT EXISTS idx_wf_rt_instance_state ON wf_rt_instance(state);

CREATE TABLE IF NOT EXISTS wf_rt_node_instance (
    instance_id      TEXT NOT NULL,
    node_id          TEXT NOT NULL,
    node_type        TEXT NOT NULL,
    state            TEXT NOT NULL,
    attempt          BIGINT NOT NULL DEFAULT 0,
    max_retry        BIGINT NOT NULL DEFAULT 0,
    retry_state_json TEXT NOT NULL,
    input_json       TEXT,
    output_json      TEXT,
    error_message    TEXT,
    started_at       TEXT,
    finished_at      TEXT,
    duration_ms      BIGINT,
    PRIMARY KEY (instance_id, node_id),
    FOREIGN KEY (instance_id) REFERENCES wf_rt_instance(instance_id)
);

CREATE INDEX IF NOT EXISTS idx_wf_rt_node_state ON wf_rt_node_instance(instance_id, state);

CREATE TABLE IF NOT EXISTS wf_rt_log (
    id               TEXT PRIMARY KEY NOT NULL,
    instance_id      TEXT NOT NULL,
    node_id          TEXT,
    level            TEXT NOT NULL,
    event_kind       TEXT NOT NULL,
    payload_json     TEXT NOT NULL,
    created_at       TEXT NOT NULL,
    FOREIGN KEY (instance_id) REFERENCES wf_rt_instance(instance_id)
);

CREATE INDEX IF NOT EXISTS idx_wf_rt_log_instance ON wf_rt_log(instance_id, created_at);
