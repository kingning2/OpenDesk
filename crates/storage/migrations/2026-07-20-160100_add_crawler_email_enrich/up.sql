-- crawler.db: two-phase email enrichment status (CHG-031).

ALTER TABLE crawler_channel ADD COLUMN email_status TEXT NOT NULL DEFAULT 'pending_enrich';
ALTER TABLE crawler_channel ADD COLUMN enrich_attempts INTEGER NOT NULL DEFAULT 0;
ALTER TABLE crawler_channel ADD COLUMN enrich_error TEXT;
ALTER TABLE crawler_channel ADD COLUMN enriched_at TEXT;

CREATE INDEX IF NOT EXISTS idx_crawler_channel_email_status
    ON crawler_channel(job_id, email_status);

UPDATE crawler_channel
SET email_status = 'found_api'
WHERE email IS NOT NULL AND email != '';

UPDATE crawler_channel
SET email_status = 'pending_enrich'
WHERE email IS NULL OR email = '';
