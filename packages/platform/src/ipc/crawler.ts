import { invoke } from "@tauri-apps/api/core";
import type {
  CrawlerIpcJobCancelRequest,
  CrawlerIpcJobCancelResponse,
  CrawlerIpcJobLogsRequest,
  CrawlerIpcJobLogsResponse,
  CrawlerIpcJobStartRequest,
  CrawlerIpcJobStartResponse,
  CrawlerIpcJobStatusRequest,
  CrawlerIpcJobStatusResponse,
} from "@desk/contracts";

/**
 * Start a crawl job (React → Rust → Python).
 *
 * @author Xiaoman
 * @created 2026-07-16
 */
export async function crawlerJobStart(
  input: CrawlerIpcJobStartRequest,
): Promise<CrawlerIpcJobStartResponse> {
  return invoke<CrawlerIpcJobStartResponse>("crawler_job_start", { request: input });
}

/**
 * Cancel a crawl job.
 *
 * @author Xiaoman
 * @created 2026-07-16
 */
export async function crawlerJobCancel(
  input: CrawlerIpcJobCancelRequest,
): Promise<CrawlerIpcJobCancelResponse> {
  return invoke<CrawlerIpcJobCancelResponse>("crawler_job_cancel", { request: input });
}

/**
 * Query crawl job status.
 *
 * @author Xiaoman
 * @created 2026-07-16
 */
export async function crawlerJobStatus(
  input: CrawlerIpcJobStatusRequest,
): Promise<CrawlerIpcJobStatusResponse> {
  return invoke<CrawlerIpcJobStatusResponse>("crawler_job_status", { request: input });
}

/**
 * Fetch crawl process logs (`logs_json` array string).
 *
 * @author Xiaoman
 * @created 2026-07-16
 */
export async function crawlerJobLogs(
  input: CrawlerIpcJobLogsRequest,
): Promise<CrawlerIpcJobLogsResponse> {
  return invoke<CrawlerIpcJobLogsResponse>("crawler_job_logs", { request: input });
}
