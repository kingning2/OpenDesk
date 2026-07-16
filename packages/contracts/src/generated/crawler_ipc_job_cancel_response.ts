export interface CrawlerIpcJobCancelResponse {
  ok: boolean;
  job_id: string;
  trace_id?: string;
}
