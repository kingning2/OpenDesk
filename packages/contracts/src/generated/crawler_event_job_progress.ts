export interface CrawlerEventJobProgress {
  event_id: string;
  occurred_at: string;
  job_id: string;
  platform: string;
  status?: string;
  message?: string;
  stop_reason?: string;
  current_keyword?: string;
  scanned_count: number;
  accepted_count: number;
  quota_used?: number;
  search_pages?: number;
  keyword_scanned?: number;
  keyword_accepted?: number;
  keywords_total?: number;
  keywords_done?: number;
  keyword_stats_json?: string;
  error_message?: string;
}
