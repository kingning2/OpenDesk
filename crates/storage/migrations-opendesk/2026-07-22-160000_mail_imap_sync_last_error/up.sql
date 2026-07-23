-- opendesk.db: IMAP sync last error column (CHG-029).

ALTER TABLE mail_imap_sync_state ADD COLUMN last_error TEXT;
