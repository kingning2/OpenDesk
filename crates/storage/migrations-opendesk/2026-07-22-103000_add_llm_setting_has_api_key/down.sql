-- SQLite cannot DROP COLUMN on older versions reliably; leave column in place on downgrade.
SELECT 1;
