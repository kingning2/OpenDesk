export interface CrawlerSidecarJobLogsResponse {
  ok: boolean;
  job_id: string;
  logs_json: string;
  trace_id?: string;
}
