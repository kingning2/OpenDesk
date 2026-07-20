export interface CrawlerEventJobFailed {
  event_id: string;
  occurred_at: string;
  job_id: string;
  platform: string;
  error_code: string;
  message: string;
}
