export interface CrawlerIpcJobStartRequest {
  trace_id?: string;
  platform: string;
  keywords: string;
  rate_limit_ms?: number;
  max_total?: number;
  year?: number;
  min_year_video_count?: number;
  exclude_countries?: string;
  batch_id?: string;
}
