export interface CrawlerEventChannelAccepted {
  event_id: string;
  occurred_at: string;
  job_id: string;
  platform: string;
  keyword: string;
  channel_id: string;
  title: string;
  country?: string;
  subscriber_count?: number;
  email?: string;
  description?: string;
  custom_url?: string;
  email_status: string;
  enrich_attempts?: number;
  enrich_error?: string;
  enriched_at?: string;
}
