export interface CrawlerIpcJobStatusResponse {
  ok: boolean;
  job_id: string;
  platform: string;
  status: string;
  stop_reason?: string;
  message?: string;
  current_keyword?: string;
  scanned_count?: number;
  accepted_count?: number;
  keyword_scanned?: number;
  keyword_accepted?: number;
  quota_used?: number;
  keywords_total?: number;
  keywords_done?: number;
  keyword_stats_json?: string;
  error_message?: string;
  trace_id?: string;
}
