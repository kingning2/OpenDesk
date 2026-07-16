export interface CrawlerEventJobProgress {
  event_id: string;
  occurred_at: string;
  job_id: string;
  platform: string;
  current_keyword?: string;
  scanned_count: number;
  accepted_count: number;
  quota_used?: number;
  search_pages?: number;
}
