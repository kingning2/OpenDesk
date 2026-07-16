export interface CrawlerEventJobLog {
  event_id: string;
  occurred_at: string;
  job_id: string;
  platform: string;
  seq: number;
  phase: string;
  level: string;
  message: string;
  keyword?: string;
  detail?: string;
}
