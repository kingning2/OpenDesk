/**
 * Crawl job hook — CSV keyword batches, IPC polling, React Flow node state.
 *
 * Progress is polled (status + logs) so the UI can drive a fixed React Flow
 * monitor without subscribing to Tauri events yet.
 */

import { useCallback, useEffect, useState } from "react";
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

function statusLabel(status: CrawlUiStatus, stopReason?: string): string {
  if (status === "idle") return "未开始";
  if (status === "queued") return "排队中";
  if (status === "running") return "爬取中";
  if (status === "cancelled") return "已取消";
  if (status === "failed") return "失败";
  if (stopReason === "quota_exceeded") return "已因配额自动停止";
  if (stopReason === "max_total_reached") return "已达数量上限";
  if (stopReason === "keywords_finished") return "已完成";
  return "已结束";
}

export function useCrawlerJob() {
  const [apiKey, setApiKey] = useState("");
  const [apiKeyLoading, setApiKeyLoading] = useState(true);
  const [batchId, setBatchId] = useState("");
  const [batches, setBatches] = useState<KeywordBatchRow[]>([]);
  const [importMessage, setImportMessage] = useState("");
  const [importing, setImporting] = useState(false);
  const [jobId, setJobId] = useState<string | null>(null);
  const [status, setStatus] = useState<CrawlUiStatus>("idle");
  const [stopReason, setStopReason] = useState("");
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
      setError(err instanceof Error ? err.message : String(err));
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
      setError(err instanceof Error ? err.message : String(err));
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
          setError(err instanceof Error ? err.message : String(err));
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
        setError("导入失败");
        return;
      }
      setBatchId(result.batch_id);
      setImportMessage(
        `已导入 ${result.inserted} 条（跳过重复 ${result.skipped_existing}，过长 ${result.skipped_too_long}）`,
      );
      await refreshBatches();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setImporting(false);
    }
  }

  async function start() {
    setError("");
    if (!apiKey.trim()) {
      setError("请先在设置中配置 YouTube API 密钥");
      return;
    }
    setBusy(true);
    setMessage("正在启动…");
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
        trace_id: crypto.randomUUID(),
      });
      if (!result.ok || !result.job_id) {
        setError("启动失败：请检查 API Key、批次与 Sidecar");
        setBusy(false);
        setStatus("failed");
        return;
      }
      setJobId(result.job_id);
      setStatus("queued");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setBusy(false);
      setStatus("failed");
    }
  }

  async function cancel() {
    if (!jobId) {
      return;
    }
    try {
      await crawlerJobCancel({ job_id: jobId });
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  const selectedBatch = batches.find((row) => row.batch_id === batchId);

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
    statusText: statusLabel(status, stopReason),
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
