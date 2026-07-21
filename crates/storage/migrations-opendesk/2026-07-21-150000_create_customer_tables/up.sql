-- opendesk.db: customer CRM tables (M1 / CHG-013).

CREATE TABLE IF NOT EXISTS customer (
    id              TEXT PRIMARY KEY NOT NULL,
    display_name    TEXT,
    email           TEXT NOT NULL,
    whatsapp_phone  TEXT,
    source_channel  TEXT NOT NULL DEFAULT 'manual',
    source_meta     TEXT,
    lifecycle_status TEXT NOT NULL DEFAULT 'new',
    outreach_stage  TEXT NOT NULL DEFAULT 'no_stage',
    quoted_price    REAL,
    quoted_currency TEXT,
    quoted_at       TEXT,
    pricing_tier    TEXT,
    cooperation_status TEXT NOT NULL DEFAULT 'none',
    package_name    TEXT,
    monthly_fee     REAL,
    contract_start  TEXT,
    contract_end    TEXT,
    notes           TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    UNIQUE(email)
);

CREATE INDEX IF NOT EXISTS idx_customer_lifecycle ON customer(lifecycle_status);
CREATE INDEX IF NOT EXISTS idx_customer_outreach ON customer(outreach_stage);
CREATE INDEX IF NOT EXISTS idx_customer_cooperation ON customer(cooperation_status);
CREATE INDEX IF NOT EXISTS idx_customer_updated ON customer(updated_at DESC);

CREATE TABLE IF NOT EXISTS quote_history (
    id              TEXT PRIMARY KEY NOT NULL,
    customer_id     TEXT NOT NULL REFERENCES customer(id) ON DELETE CASCADE,
    old_price       REAL,
    new_price       REAL,
    currency        TEXT,
    old_tier        TEXT,
    new_tier        TEXT,
    reason          TEXT,
    changed_by      TEXT,
    created_at      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_quote_history_customer ON quote_history(customer_id, created_at DESC);

CREATE TABLE IF NOT EXISTS customer_timeline (
    id              TEXT PRIMARY KEY NOT NULL,
    customer_id     TEXT NOT NULL REFERENCES customer(id) ON DELETE CASCADE,
    entry_type      TEXT NOT NULL,
    ref_id          TEXT,
    summary         TEXT NOT NULL,
    metadata_json   TEXT,
    created_at      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_timeline_customer ON customer_timeline(customer_id, created_at DESC);

CREATE TABLE IF NOT EXISTS cooperation_audit (
    id              TEXT PRIMARY KEY NOT NULL,
    customer_id     TEXT NOT NULL REFERENCES customer(id) ON DELETE CASCADE,
    field_name      TEXT NOT NULL,
    old_value       TEXT,
    new_value       TEXT,
    changed_by      TEXT,
    created_at      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_cooperation_audit_customer ON cooperation_audit(customer_id, created_at DESC);
