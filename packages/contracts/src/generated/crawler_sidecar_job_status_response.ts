export interface CrawlerSidecarJobStatusResponse {
  ok: boolean;
  job_id: string;
  platform: string;
  status: string;
  stop_reason?: string;
  scanned_count?: number;
  accepted_count?: number;
  trace_id?: string;
}
