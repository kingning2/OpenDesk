-- opendesk.db: email-agent real-data migration support (CHG-20260722-040).

ALTER TABLE customer ADD COLUMN extra_json TEXT;
ALTER TABLE customer ADD COLUMN source_ref TEXT;

ALTER TABLE mail_message ADD COLUMN from_address TEXT;
ALTER TABLE mail_message ADD COLUMN source_ref TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_mail_message_source_ref
    ON mail_message(source_ref)
    WHERE source_ref IS NOT NULL;

CREATE TABLE IF NOT EXISTS mail_imap_sync_state (
    account_id      TEXT NOT NULL REFERENCES mail_account(id) ON DELETE CASCADE,
    folder          TEXT NOT NULL DEFAULT 'INBOX',
    uidvalidity     INTEGER NOT NULL DEFAULT 0,
    highest_modseq  TEXT NOT NULL DEFAULT '0',
    last_uid        INTEGER NOT NULL DEFAULT 0,
    last_sync_at    TEXT,
    full_synced     INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, folder)
);

CREATE TABLE IF NOT EXISTS mail_pending_send (
    id              TEXT PRIMARY KEY NOT NULL,
    account_id      TEXT REFERENCES mail_account(id) ON DELETE SET NULL,
    recipients_json TEXT NOT NULL,
    subject         TEXT NOT NULL,
    body_text       TEXT NOT NULL,
    body_html       TEXT,
    status          TEXT NOT NULL DEFAULT 'pending',
    scheduled_at    TEXT,
    source_ref      TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_mail_pending_send_source_ref
    ON mail_pending_send(source_ref)
    WHERE source_ref IS NOT NULL;

CREATE TABLE IF NOT EXISTS legacy_json_doc (
    id              TEXT PRIMARY KEY NOT NULL,
    kind            TEXT NOT NULL,
    payload_json    TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_legacy_json_doc_kind ON legacy_json_doc(kind);
