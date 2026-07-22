DROP TABLE IF EXISTS legacy_json_doc;
DROP TABLE IF EXISTS mail_pending_send;
DROP TABLE IF EXISTS mail_imap_sync_state;

DROP INDEX IF EXISTS idx_mail_message_source_ref;

-- SQLite cannot DROP COLUMN; down migration is best-effort for dev rollback.
-- Recreate mail_message/customer without extension columns if full rollback is required.
