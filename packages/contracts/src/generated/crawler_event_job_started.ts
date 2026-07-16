export interface CrawlerEventJobStarted {
  event_id: string;
  occurred_at: string;
  job_id: string;
  platform: string;
  keywords?: string;
}
