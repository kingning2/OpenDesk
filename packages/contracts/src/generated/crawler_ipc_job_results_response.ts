export interface CrawlerIpcJobResultsResponse {
  ok: boolean;
  job_id: string;
  results_json: string;
  trace_id?: string;
}
