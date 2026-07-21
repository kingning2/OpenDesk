/**
 * Crawl job hook — CSV keyword batches + Tauri Event 推送（无轮询）.
 *
 * 进度 / 日志 / 频道行由 Rust `crawler:*` Event 推送；status/logs/results IPC
 * 仍保留供调试，UI 主路径不再 setInterval。
 *
 * @author coisini
 * @created 2026-07-20
 */

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  crawlerJobCancel,
  crawlerJobStart,
  crawlerKeywordsBatches,
  crawlerKeywordsImport,
  type KeywordBatchRow,
} from "@desk/platform/ipc/crawler";
import { listenCrawlerEvents } from "@desk/platform/ipc/crawler-events";
import { crawlerYoutubeApiKeyGet } from "@desk/platform/ipc/crawler-settings";
import type {
  CrawlerEventChannelAccepted,
  CrawlerEventJobCompleted,
  CrawlerEventJobFailed,
  CrawlerEventJobLog,
  CrawlerEventJobProgress,
} from "@desk/contracts";

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

/** One accepted channel row from `crawler_channel` SQLite / Event. */
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
  email_status?: string;
  enrich_attempts?: number;
  enrich_error?: string;
  enriched_at?: string;
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
 * @author coisini
 * @created 2026-07-20
 *
 * @param err - 捕获的错误
 * @returns 展示文案
 */
function toDisplayError(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

/**
 * Map Event progress payload into UI state fields.
 *
 * @author coisini
 * @created 2026-07-21
 */
function applyProgress(
  payload: CrawlerEventJobProgress,
  setters: {
    setStatus: (value: CrawlUiStatus) => void;
    setStopReason: (value: string) => void;
    setMessage: (value: string) => void;
    setCurrentKeyword: (value: string) => void;
    setKeywordAccepted: (value: number) => void;
    setKeywordScanned: (value: number) => void;
    setAcceptedCount: (value: number) => void;
    setScannedCount: (value: number) => void;
    setQuotaUsed: (value: number) => void;
    setKeywordsTotal: (value: number) => void;
    setKeywordsDone: (value: number) => void;
    setKeywordStats: (value: KeywordStatRow[]) => void;
    setError: (value: string) => void;
    setBusy: (value: boolean) => void;
  },
) {
  const nextStatus = (payload.status || "running") as CrawlUiStatus;
  setters.setStatus(nextStatus);
  setters.setStopReason(payload.stop_reason ?? "");
  setters.setMessage(payload.message ?? "");
  setters.setCurrentKeyword(payload.current_keyword ?? "");
  setters.setKeywordAccepted(payload.keyword_accepted ?? 0);
  setters.setKeywordScanned(payload.keyword_scanned ?? 0);
  setters.setAcceptedCount(payload.accepted_count ?? 0);
  setters.setScannedCount(payload.scanned_count ?? 0);
  setters.setQuotaUsed(payload.quota_used ?? 0);
  setters.setKeywordsTotal(payload.keywords_total ?? 0);
  setters.setKeywordsDone(payload.keywords_done ?? 0);
  if (payload.error_message) {
    setters.setError(payload.error_message);
  }
  try {
    const parsed = JSON.parse(payload.keyword_stats_json ?? "[]") as KeywordStatRow[];
    setters.setKeywordStats(Array.isArray(parsed) ? parsed : []);
  } catch {
    setters.setKeywordStats([]);
  }
  if (
    nextStatus === "completed" ||
    nextStatus === "failed" ||
    nextStatus === "cancelled"
  ) {
    setters.setBusy(false);
  }
}

/**
 * Map channel.accepted Event into a results row.
 *
 * @author coisini
 * @created 2026-07-21
 */
function channelFromAccepted(payload: CrawlerEventChannelAccepted): ChannelResultRow {
  return {
    keyword: payload.keyword,
    platform: payload.platform,
    channel_id: payload.channel_id,
    title: payload.title,
    country: payload.country,
    subscriber_count: payload.subscriber_count,
    email: payload.email,
    description: payload.description,
    custom_url: payload.custom_url,
    email_status: payload.email_status,
    enrich_attempts: payload.enrich_attempts,
    enrich_error: payload.enrich_error,
    enriched_at: payload.enriched_at,
  };
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
  const jobIdRef = useRef<string | null>(null);

  useEffect(() => {
    jobIdRef.current = jobId;
  }, [jobId]);

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

  // Subscribe once on mount so events are not lost between start() and setJobId.
  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    void listenCrawlerEvents({
      onProgress: (payload: CrawlerEventJobProgress) => {
        const activeJobId = jobIdRef.current;
        if (cancelled || !activeJobId || payload.job_id !== activeJobId) {
          return;
        }
        applyProgress(payload, {
          setStatus,
          setStopReason,
          setMessage,
          setCurrentKeyword,
          setKeywordAccepted,
          setKeywordScanned,
          setAcceptedCount,
          setScannedCount,
          setQuotaUsed,
          setKeywordsTotal,
          setKeywordsDone,
          setKeywordStats,
          setError,
          setBusy,
        });
      },
      onLog: (payload: CrawlerEventJobLog) => {
        const activeJobId = jobIdRef.current;
        if (cancelled || !activeJobId || payload.job_id !== activeJobId) {
          return;
        }
        setLogs((prev) => {
          if (prev.some((row) => row.event_id === payload.event_id || row.seq === payload.seq)) {
            return prev;
          }
          return [
            ...prev,
            {
              event_id: payload.event_id,
              occurred_at: payload.occurred_at,
              job_id: payload.job_id,
              platform: payload.platform,
              seq: payload.seq,
              phase: payload.phase,
              level: payload.level,
              message: payload.message,
              keyword: payload.keyword,
              detail: payload.detail,
            },
          ];
        });
      },
      onChannelAccepted: (payload: CrawlerEventChannelAccepted) => {
        const activeJobId = jobIdRef.current;
        if (cancelled || !activeJobId || payload.job_id !== activeJobId) {
          return;
        }
        const row = channelFromAccepted(payload);
        setChannelResults((prev) => {
          const index = prev.findIndex((item) => item.channel_id === row.channel_id);
          if (index >= 0) {
            const next = prev.slice();
            next[index] = row;
            return next;
          }
          return [...prev, row];
        });
      },
      onCompleted: (payload: CrawlerEventJobCompleted) => {
        const activeJobId = jobIdRef.current;
        if (cancelled || !activeJobId || payload.job_id !== activeJobId) {
          return;
        }
        const nextStatus =
          payload.stop_reason === "cancelled" ? "cancelled" : ("completed" as CrawlUiStatus);
        setStatus(nextStatus);
        setStopReason(payload.stop_reason);
        setAcceptedCount(payload.accepted_count);
        setScannedCount(payload.scanned_count);
        setQuotaUsed(payload.quota_used ?? 0);
        setBusy(false);
      },
      onFailed: (payload: CrawlerEventJobFailed) => {
        const activeJobId = jobIdRef.current;
        if (cancelled || !activeJobId || payload.job_id !== activeJobId) {
          return;
        }
        setStatus("failed");
        setError(payload.message);
        setBusy(false);
      },
    })
      .then((dispose) => {
        if (cancelled) {
          dispose();
          return;
        }
        unlisten = dispose;
      })
      .catch((err) => {
        if (!cancelled) {
          setError(toDisplayError(err));
        }
      });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

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
    jobIdRef.current = null;
    setJobId(null);
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
      // Set ref before state so any in-flight Event matches immediately.
      jobIdRef.current = result.job_id;
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
