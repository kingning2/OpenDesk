import { invokeIpc } from "./invoke";
import type {
  CrawlerIpcChannelListRequest,
  CrawlerIpcChannelListResponse,
  CrawlerIpcChannelUpdateRequest,
  CrawlerIpcChannelUpdateResponse,
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
  CrawlerIpcKeywordsGenerateRequest,
  CrawlerIpcKeywordsGenerateResponse,
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
  return invokeIpc<CrawlerIpcJobStartResponse>("crawler_job_start", { request: input });
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
  return invokeIpc<CrawlerIpcJobCancelResponse>("crawler_job_cancel", { request: input });
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
  return invokeIpc<CrawlerIpcJobStatusResponse>("crawler_job_status", { request: input });
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
  return invokeIpc<CrawlerIpcJobLogsResponse>("crawler_job_logs", { request: input });
}

/**
 * List accepted channels for a job (`results_json` array string).
 */
export async function crawlerJobResults(
  input: CrawlerIpcJobResultsRequest,
): Promise<CrawlerIpcJobResultsResponse> {
  return invokeIpc<CrawlerIpcJobResultsResponse>("crawler_job_results", { request: input });
}

/**
 * List persisted crawler channels with optional filters and pagination.
 *
 * @author coisini
 * @created 2026-07-22
 */
export async function crawlerChannelList(
  input: CrawlerIpcChannelListRequest = {},
): Promise<{ items: CrawlerChannelListRow[]; total: number }> {
  const response = await invokeIpc<CrawlerIpcChannelListResponse>("crawler_channel_list", {
    request: input,
  });
  try {
    const parsed = JSON.parse(response.channels_json ?? "[]") as CrawlerChannelListRow[];
    return {
      items: Array.isArray(parsed) ? parsed : [],
      total: response.total ?? 0,
    };
  } catch {
    return { items: [], total: 0 };
  }
}

export interface CrawlerChannelListRow {
  id: number;
  job_id: string;
  keyword: string;
  platform: string;
  channel_id: string;
  title: string;
  country?: string;
  subscriber_count?: number;
  email?: string;
  verified_email?: string;
  description?: string;
  custom_url?: string;
  email_status?: string;
  enrich_attempts?: number;
  enrich_error?: string;
  enriched_at?: string;
}

/**
 * Save human-verified email for one crawler channel row.
 *
 * @author coisini
 * @created 2026-07-22
 */
export async function crawlerChannelUpdate(
  input: CrawlerIpcChannelUpdateRequest,
): Promise<CrawlerIpcChannelUpdateResponse> {
  return invokeIpc<CrawlerIpcChannelUpdateResponse>("crawler_channel_update", { request: input });
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
  return invokeIpc<CrawlerIpcKeywordsImportResponse>("crawler_keywords_import", { request: input });
}

/**
 * 用已配置 LLM 按方向 / 语言 / 数量生成关键词并写入新 batch。
 *
 * @author coisini
 * @created 2026-07-23
 *
 * @param input - 方向（英文逗号分隔）、语言、每语言数量
 * @returns 新 batch 与生成统计
 */
export async function crawlerKeywordsGenerate(
  input: CrawlerIpcKeywordsGenerateRequest,
): Promise<CrawlerIpcKeywordsGenerateResponse> {
  return invokeIpc<CrawlerIpcKeywordsGenerateResponse>("crawler_keywords_generate", {
    request: input,
  });
}

/**
 * List keyword import batches.
 */
export async function crawlerKeywordsBatches(): Promise<KeywordBatchRow[]> {
  const response = await invokeIpc<CrawlerIpcKeywordsBatchesResponse>("crawler_keywords_batches");
  try {
    const parsed = JSON.parse(response.batches_json ?? "[]") as KeywordBatchRow[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}
