-- opendesk.db: local read state for inbox messages.

ALTER TABLE mail_message ADD COLUMN is_read INTEGER NOT NULL DEFAULT 0;

-- Outbound messages are always read from the sender's perspective.
UPDATE mail_message SET is_read = 1 WHERE direction = 'outbound';
