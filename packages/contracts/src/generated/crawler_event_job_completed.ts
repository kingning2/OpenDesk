export interface CrawlerEventJobCompleted {
  event_id: string;
  occurred_at: string;
  job_id: string;
  platform: string;
  stop_reason: string;
  scanned_count: number;
  accepted_count: number;
  quota_used?: number;
  duration_ms?: number;
}
