export interface CrawlerEventChannelEmailEnriched {
  event_id: string;
  occurred_at: string;
  job_id: string;
  platform: string;
  channel_id: string;
  email?: string;
  email_status: string;
  enrich_attempts?: number;
  enrich_error?: string;
  enriched_at?: string;
}
