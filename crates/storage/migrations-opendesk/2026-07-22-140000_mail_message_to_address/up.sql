-- Add recipient address for free-form outbound mail (no customer required).
ALTER TABLE mail_message ADD COLUMN to_address TEXT;
