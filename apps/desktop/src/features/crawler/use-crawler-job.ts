/**
 * Crawl job hook — CSV keyword batches, IPC polling, React Flow node state.
 *
 * Progress is polled (status + logs) so the UI can drive a fixed React Flow
 * monitor without subscribing to Tauri events yet.
 * 后端 job `message` / IPC 错误已按 `locale` 翻译，前端直接展示。
 *
 * @author Xiaoman
 * @created 2026-07-20
 */

import { useCallback, useEffect, useMemo, useState } from "react";
import {
  crawlerJobCancel,
  crawlerJobLogs,
  crawlerJobResults,
  crawlerJobStart,
  crawlerJobStatus,
  crawlerKeywordsBatches,
  crawlerKeywordsImport,
  type KeywordBatchRow,
} from "@desk/platform/ipc/crawler";
import { crawlerYoutubeApiKeyGet } from "@desk/platform/ipc/crawler-settings";

import { useI18n, useT } from "../../i18n";

export interface KeywordStatRow {
  keyword: string;
  scanned: number;
  accepted: number;
}

export interface CrawlerLogRow {
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

/** One accepted channel row from `crawler_channel` SQLite. */
export interface ChannelResultRow {
  keyword: string;
  platform: string;
  channel_id: string;
  title: string;
  country?: string;
  subscriber_count?: number;
  email?: string;
  description?: string;
  custom_url?: string;
}

export type CrawlUiStatus =
  | "idle"
  | "queued"
  | "running"
  | "completed"
  | "failed"
  | "cancelled";

function statusLabel(
  status: CrawlUiStatus,
  t: (key: string, params?: Record<string, string | number>) => string,
  stopReason?: string,
): string {
  if (status === "idle") return t("crawler.status.idle");
  if (status === "queued") return t("crawler.status.queued");
  if (status === "running") return t("crawler.status.running");
  if (status === "cancelled") return t("crawler.status.cancelled");
  if (status === "failed") return t("crawler.status.failed");
  if (stopReason === "quota_exceeded") return t("crawler.status.quotaStop");
  if (stopReason === "max_total_reached") return t("crawler.status.maxTotal");
  if (stopReason === "keywords_finished") return t("crawler.status.keywordsFinished");
  return t("crawler.status.ended");
}

/**
 * 将 IPC 错误转为展示字符串（后端已翻译）。
 *
 * @author Xiaoman
 * @created 2026-07-20
 *
 * @param err - 捕获的错误
 * @returns 展示文案
 */
function toDisplayError(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

export function useCrawlerJob() {
  const t = useT();
  const { locale } = useI18n();
  const [apiKey, setApiKey] = useState("");
  const [apiKeyLoading, setApiKeyLoading] = useState(true);
  const [batchId, setBatchId] = useState("");
  const [batches, setBatches] = useState<KeywordBatchRow[]>([]);
  const [importMessage, setImportMessage] = useState("");
  const [importing, setImporting] = useState(false);
  const [jobId, setJobId] = useState<string | null>(null);
  const [status, setStatus] = useState<CrawlUiStatus>("idle");
  const [stopReason, setStopReason] = useState("");
  /** 后端已按 locale 翻译的 job message。 */
  const [message, setMessage] = useState("");
  const [currentKeyword, setCurrentKeyword] = useState("");
  const [keywordAccepted, setKeywordAccepted] = useState(0);
  const [keywordScanned, setKeywordScanned] = useState(0);
  const [acceptedCount, setAcceptedCount] = useState(0);
  const [scannedCount, setScannedCount] = useState(0);
  const [quotaUsed, setQuotaUsed] = useState(0);
  const [keywordsTotal, setKeywordsTotal] = useState(0);
  const [keywordsDone, setKeywordsDone] = useState(0);
  const [keywordStats, setKeywordStats] = useState<KeywordStatRow[]>([]);
  const [logs, setLogs] = useState<CrawlerLogRow[]>([]);
  const [channelResults, setChannelResults] = useState<ChannelResultRow[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  const refreshApiKey = useCallback(async () => {
    setApiKeyLoading(true);
    try {
      const response = await crawlerYoutubeApiKeyGet();
      setApiKey(response.api_key ?? "");
    } catch (err) {
      setError(toDisplayError(err));
    } finally {
      setApiKeyLoading(false);
    }
  }, []);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void refreshApiKey();
    }, 0);
    return () => {
      window.clearTimeout(timer);
    };
  }, [refreshApiKey]);

  const refreshBatches = useCallback(async () => {
    try {
      const items = await crawlerKeywordsBatches();
      setBatches(items);
      if (!batchId && items.length > 0) {
        setBatchId(items[0].batch_id);
      }
    } catch (err) {
      setError(toDisplayError(err));
    }
  }, [batchId]);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void refreshBatches();
    }, 0);
    return () => {
      window.clearTimeout(timer);
    };
  }, [refreshBatches]);

  useEffect(() => {
    if (!jobId) {
      return;
    }
    let cancelled = false;

    async function poll() {
      try {
        const [statusRes, logsRes, resultsRes] = await Promise.all([
          crawlerJobStatus({ job_id: jobId! }),
          crawlerJobLogs({ job_id: jobId! }),
          crawlerJobResults({ job_id: jobId! }),
        ]);
        if (cancelled) {
          return;
        }
        const nextStatus = (statusRes.status || "running") as CrawlUiStatus;
        setStatus(nextStatus);
        setStopReason(statusRes.stop_reason ?? "");
        setMessage(statusRes.message ?? "");
        setCurrentKeyword(statusRes.current_keyword ?? "");
        setKeywordAccepted(statusRes.keyword_accepted ?? 0);
        setKeywordScanned(statusRes.keyword_scanned ?? 0);
        setAcceptedCount(statusRes.accepted_count ?? 0);
        setScannedCount(statusRes.scanned_count ?? 0);
        setQuotaUsed(statusRes.quota_used ?? 0);
        setKeywordsTotal(statusRes.keywords_total ?? 0);
        setKeywordsDone(statusRes.keywords_done ?? 0);
        if (statusRes.error_message) {
          setError(statusRes.error_message);
        }
        try {
          const parsed = JSON.parse(statusRes.keyword_stats_json ?? "[]") as KeywordStatRow[];
          setKeywordStats(Array.isArray(parsed) ? parsed : []);
        } catch {
          setKeywordStats([]);
        }
        try {
          const parsed = JSON.parse(logsRes.logs_json ?? "[]") as CrawlerLogRow[];
          setLogs(Array.isArray(parsed) ? parsed : []);
        } catch {
          setLogs([]);
        }
        try {
          const parsed = JSON.parse(resultsRes.results_json ?? "[]") as ChannelResultRow[];
          setChannelResults(Array.isArray(parsed) ? parsed : []);
        } catch {
          setChannelResults([]);
        }
        if (
          nextStatus === "completed" ||
          nextStatus === "failed" ||
          nextStatus === "cancelled"
        ) {
          setBusy(false);
        }
      } catch (err) {
        if (!cancelled) {
          setError(toDisplayError(err));
          setBusy(false);
          setStatus("failed");
        }
      }
    }

    void poll();
    const timer = window.setInterval(() => {
      void poll();
    }, 800);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, [jobId]);

  async function importCsvFile(file: File) {
    setError("");
    setImportMessage("");
    setImporting(true);
    try {
      const csvContent = await file.text();
      const result = await crawlerKeywordsImport({
        csv_content: csvContent,
        trace_id: crypto.randomUUID(),
      });
      if (!result.ok) {
        setError(t("crawler.importFailed"));
        return;
      }
      setBatchId(result.batch_id);
      setImportMessage(
        t("crawler.importResult", {
          inserted: result.inserted,
          skipped: result.skipped_existing,
          tooLong: result.skipped_too_long,
        }),
      );
      await refreshBatches();
    } catch (err) {
      setError(toDisplayError(err));
    } finally {
      setImporting(false);
    }
  }

  async function start() {
    setError("");
    if (!apiKey.trim()) {
      setError(t("crawler.needApiKey"));
      return;
    }
    setBusy(true);
    setMessage(t("crawler.starting"));
    setStopReason("");
    setCurrentKeyword("");
    setKeywordStats([]);
    setLogs([]);
    setChannelResults([]);
    setAcceptedCount(0);
    setScannedCount(0);
    setKeywordAccepted(0);
    setKeywordScanned(0);
    setQuotaUsed(0);
    setKeywordsTotal(0);
    setKeywordsDone(0);
    try {
      const result = await crawlerJobStart({
        platform: "youtube",
        batch_id: batchId,
        api_key: apiKey,
        rate_limit_ms: 400,
        locale,
        trace_id: crypto.randomUUID(),
      });
      if (!result.ok || !result.job_id) {
        setBusy(false);
        setStatus("failed");
        setError(t("crawler.startFailed"));
        return;
      }
      setJobId(result.job_id);
      setStatus("queued");
    } catch (err) {
      setBusy(false);
      setStatus("failed");
      setError(toDisplayError(err));
    }
  }

  async function cancel() {
    if (!jobId) {
      return;
    }
    try {
      await crawlerJobCancel({ job_id: jobId });
    } catch (err) {
      setError(toDisplayError(err));
    }
  }

  const selectedBatch = batches.find((row) => row.batch_id === batchId);
  const statusText = useMemo(
    () => statusLabel(status, t, stopReason),
    [status, t, stopReason],
  );

  return {
    apiKey,
    apiKeyConfigured: Boolean(apiKey.trim()),
    apiKeyLoading,
    refreshApiKey,
    batchId,
    setBatchId,
    batches,
    selectedBatch,
    importMessage,
    importing,
    importCsvFile,
    refreshBatches,
    jobId,
    status,
    statusText,
    stopReason,
    message,
    currentKeyword,
    keywordAccepted,
    keywordScanned,
    acceptedCount,
    scannedCount,
    quotaUsed,
    keywordsTotal,
    keywordsDone,
    keywordStats,
    logs,
    channelResults,
    busy,
    error,
    start,
    cancel,
  };
}
