-- opendesk.db: recipient open status synced from email-read API (outbound tracking).

ALTER TABLE mail_message ADD COLUMN opened_at TEXT;
ALTER TABLE mail_message ADD COLUMN open_count INTEGER NOT NULL DEFAULT 0;
