export interface CrawlerSidecarJobCancelResponse {
  ok: boolean;
  job_id: string;
  trace_id?: string;
}
