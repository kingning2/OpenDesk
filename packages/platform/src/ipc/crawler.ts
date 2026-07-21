import { invoke } from "@tauri-apps/api/core";
import type {
  CrawlerIpcJobCancelRequest,
  CrawlerIpcJobCancelResponse,
  CrawlerIpcJobLogsRequest,
  CrawlerIpcJobLogsResponse,
  CrawlerIpcJobResultsRequest,
  CrawlerIpcJobResultsResponse,
  CrawlerIpcJobStartRequest,
  CrawlerIpcJobStartResponse,
  CrawlerIpcJobStatusRequest,
  CrawlerIpcJobStatusResponse,
  CrawlerIpcKeywordsBatchesResponse,
  CrawlerIpcKeywordsImportRequest,
  CrawlerIpcKeywordsImportResponse,
} from "@desk/contracts";

/**
 * Start a crawl job (React → Rust in-process crawler).
 *
 * @author coisini
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
 * @author coisini
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
 * @author coisini
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
 * @author coisini
 * @created 2026-07-16
 */
export async function crawlerJobLogs(
  input: CrawlerIpcJobLogsRequest,
): Promise<CrawlerIpcJobLogsResponse> {
  return invoke<CrawlerIpcJobLogsResponse>("crawler_job_logs", { request: input });
}

/**
 * List accepted channels for a job (`results_json` array string).
 */
export async function crawlerJobResults(
  input: CrawlerIpcJobResultsRequest,
): Promise<CrawlerIpcJobResultsResponse> {
  return invoke<CrawlerIpcJobResultsResponse>("crawler_job_results", { request: input });
}

export interface KeywordBatchRow {
  batch_id: string;
  keyword_count: number;
}

/**
 * Import keywords from CSV text into Rust SQLite.
 */
export async function crawlerKeywordsImport(
  input: CrawlerIpcKeywordsImportRequest,
): Promise<CrawlerIpcKeywordsImportResponse> {
  return invoke<CrawlerIpcKeywordsImportResponse>("crawler_keywords_import", { request: input });
}

/**
 * List keyword import batches.
 */
export async function crawlerKeywordsBatches(): Promise<KeywordBatchRow[]> {
  const response = await invoke<CrawlerIpcKeywordsBatchesResponse>("crawler_keywords_batches");
  try {
    const parsed = JSON.parse(response.batches_json ?? "[]") as KeywordBatchRow[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}
