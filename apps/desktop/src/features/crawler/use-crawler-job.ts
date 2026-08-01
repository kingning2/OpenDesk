/**
 * Crawl job hook — CSV / AI keywords + Event 推送 + 任务结束后自动生成下一批。
 *
 * @author coisini
 * @created 2026-07-20
 */

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  crawlerJobCancel,
  crawlerJobStart,
  crawlerKeywordsBatches,
  crawlerKeywordsGenerate,
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

/**
 * 单个关键词进度行。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
export interface KeywordStatRow {
  keyword: string;
  scanned: number;
  accepted: number;
}

/**
 * 采集过程日志行。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
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

/**
 * 已收录频道行（来自 Event / SQLite）。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
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

/**
 * UI 任务状态。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
export type CrawlUiStatus =
  | "idle"
  | "queued"
  | "running"
  | "completed"
  | "failed"
  | "cancelled";

/**
 * AI 生成阶段状态（工作流节点）。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
export type GeneratePhase = "idle" | "running" | "done" | "error";

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
    setAcceptedCount: (value: number) => void;
    setScannedCount: (value: number) => void;
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
  setters.setAcceptedCount(payload.accepted_count ?? 0);
  setters.setScannedCount(payload.scanned_count ?? 0);
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

/**
 * 采集任务 hook：手动 CSV / AI 生成 + 结束后自动连跑。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @returns 采集状态、AI 参数与启停动作
 */
export function useCrawlerJob() {
  const t = useT();
  const { locale } = useI18n();
  const [apiKey, setApiKey] = useState("");
  const [apiKeyLoading, setApiKeyLoading] = useState(true);
  const [batchId, setBatchId] = useState("");
  const [batches, setBatches] = useState<KeywordBatchRow[]>([]);
  const [importMessage, setImportMessage] = useState("");
  const [importing, setImporting] = useState(false);
  const [directions, setDirections] = useState("");
  const [languages, setLanguages] = useState("en,zh");
  const [countPerLanguage, setCountPerLanguage] = useState(20);
  const [autoLoop, setAutoLoop] = useState(true);
  const [generatePhase, setGeneratePhase] = useState<GeneratePhase>("idle");
  const [generateMessage, setGenerateMessage] = useState("");
  const [lastGeneratedKeywords, setLastGeneratedKeywords] = useState<string[]>([]);
  const [jobId, setJobId] = useState<string | null>(null);
  const [status, setStatus] = useState<CrawlUiStatus>("idle");
  const [stopReason, setStopReason] = useState("");
  const [message, setMessage] = useState("");
  const [currentKeyword, setCurrentKeyword] = useState("");
  const [acceptedCount, setAcceptedCount] = useState(0);
  const [scannedCount, setScannedCount] = useState(0);
  const [keywordsTotal, setKeywordsTotal] = useState(0);
  const [keywordsDone, setKeywordsDone] = useState(0);
  const [keywordStats, setKeywordStats] = useState<KeywordStatRow[]>([]);
  const [logs, setLogs] = useState<CrawlerLogRow[]>([]);
  const [channelResults, setChannelResults] = useState<ChannelResultRow[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const jobIdRef = useRef<string | null>(null);
  const autoLoopRef = useRef(true);
  const aiLoopEnabledRef = useRef(false);
  const stopRequestedRef = useRef(false);
  const loopInFlightRef = useRef(false);
  const startCrawlRef = useRef<(nextBatchId?: string) => Promise<void>>(async () => undefined);
  const generateAndStartRef = useRef<() => Promise<boolean>>(async () => false);

  useEffect(() => {
    jobIdRef.current = jobId;
  }, [jobId]);

  useEffect(() => {
    autoLoopRef.current = autoLoop;
  }, [autoLoop]);

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

  const aiReady = useMemo(() => {
    return (
      directions.trim().length > 0 &&
      languages.trim().length > 0 &&
      countPerLanguage > 0
    );
  }, [countPerLanguage, directions, languages]);

  const generateBatch = useCallback(async (): Promise<string | null> => {
    setGeneratePhase("running");
    setGenerateMessage(t("crawler.generatingKeywords"));
    setError("");
    try {
      const result = await crawlerKeywordsGenerate({
        directions: directions.trim(),
        languages: languages.trim(),
        count_per_language: countPerLanguage,
        trace_id: crypto.randomUUID(),
      });
      if (!result.ok || !result.batch_id) {
        setGeneratePhase("error");
        setGenerateMessage(t("crawler.generateFailed"));
        setError(t("crawler.generateFailed"));
        return null;
      }
      let keywords: string[] = [];
      try {
        const parsed = JSON.parse(result.keywords_json ?? "[]") as string[];
        keywords = Array.isArray(parsed) ? parsed : [];
      } catch {
        keywords = [];
      }
      setLastGeneratedKeywords(keywords);
      setBatchId(result.batch_id);
      setGeneratePhase("done");
      setGenerateMessage(
        t("crawler.generateResult", {
          inserted: result.inserted,
          requested: result.requested,
        }),
      );
      setImportMessage(
        t("crawler.generateResult", {
          inserted: result.inserted,
          requested: result.requested,
        }),
      );
      await refreshBatches();
      return result.batch_id;
    } catch (err) {
      setGeneratePhase("error");
      const text = toDisplayError(err);
      setGenerateMessage(text);
      setError(text);
      return null;
    }
  }, [countPerLanguage, directions, languages, refreshBatches, t]);

  const startCrawl = useCallback(
    async (nextBatchId?: string) => {
      setError("");
      if (!apiKey.trim()) {
        setError(t("crawler.needApiKey"));
        return;
      }
      const activeBatchId = (nextBatchId ?? batchId).trim();
      if (!activeBatchId) {
        setError(t("crawler.needBatchOrAi"));
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
      setKeywordsTotal(0);
      setKeywordsDone(0);
      jobIdRef.current = null;
      setJobId(null);
      try {
        const result = await crawlerJobStart({
          platform: "youtube",
          batch_id: activeBatchId,
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
        jobIdRef.current = result.job_id;
        setJobId(result.job_id);
        setStatus("queued");
      } catch (err) {
        setBusy(false);
        setStatus("failed");
        setError(toDisplayError(err));
      }
    },
    [apiKey, batchId, locale, t],
  );

  const generateAndStart = useCallback(async (): Promise<boolean> => {
    if (!aiReady) {
      setError(t("crawler.needAiFields"));
      return false;
    }
    setBusy(true);
    const nextBatchId = await generateBatch();
    if (!nextBatchId) {
      setBusy(false);
      return false;
    }
    if (stopRequestedRef.current) {
      setBusy(false);
      setGenerateMessage(t("crawler.status.cancelled"));
      return false;
    }
    await startCrawl(nextBatchId);
    return true;
  }, [aiReady, generateBatch, startCrawl, t]);

  useEffect(() => {
    startCrawlRef.current = startCrawl;
  }, [startCrawl]);

  useEffect(() => {
    generateAndStartRef.current = generateAndStart;
  }, [generateAndStart]);

  // Subscribe once on mount so events are not lost between start() and setJobId.
  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    void listenCrawlerEvents({
      onProgress: (payload: CrawlerEventJobProgress) => {
        if (cancelled) {
          return;
        }
        let activeJobId = jobIdRef.current;
        if (!activeJobId) {
          jobIdRef.current = payload.job_id;
          setJobId(payload.job_id);
          activeJobId = payload.job_id;
        }
        if (payload.job_id !== activeJobId) {
          return;
        }
        applyProgress(payload, {
          setStatus,
          setStopReason,
          setMessage,
          setCurrentKeyword,
          setAcceptedCount,
          setScannedCount,
          setKeywordsTotal,
          setKeywordsDone,
          setKeywordStats,
          setError,
          setBusy,
        });
      },
      onLog: (payload: CrawlerEventJobLog) => {
        if (cancelled) {
          return;
        }
        let activeJobId = jobIdRef.current;
        if (!activeJobId) {
          jobIdRef.current = payload.job_id;
          setJobId(payload.job_id);
          activeJobId = payload.job_id;
        }
        if (payload.job_id !== activeJobId) {
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
        if (cancelled) {
          return;
        }
        let activeJobId = jobIdRef.current;
        if (!activeJobId) {
          jobIdRef.current = payload.job_id;
          setJobId(payload.job_id);
          activeJobId = payload.job_id;
        }
        if (payload.job_id !== activeJobId) {
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
        if (cancelled) {
          return;
        }
        let activeJobId = jobIdRef.current;
        if (!activeJobId) {
          jobIdRef.current = payload.job_id;
          setJobId(payload.job_id);
          activeJobId = payload.job_id;
        }
        if (payload.job_id !== activeJobId) {
          return;
        }
        const nextStatus =
          payload.stop_reason === "cancelled" ? "cancelled" : ("completed" as CrawlUiStatus);
        setStatus(nextStatus);
        setStopReason(payload.stop_reason);
        setAcceptedCount(payload.accepted_count);
        setScannedCount(payload.scanned_count);

        if (payload.stop_reason === "cancelled") {
          setBusy(false);
          autoLoopRef.current = false;
          setAutoLoop(false);
          return;
        }
        if (!autoLoopRef.current || !aiLoopEnabledRef.current || loopInFlightRef.current) {
          setBusy(false);
          return;
        }
        // Keep busy across generate→start so Stop stays available and UI shows the loop.
        setBusy(true);
        loopInFlightRef.current = true;
        void (async () => {
          try {
            await generateAndStartRef.current();
          } finally {
            loopInFlightRef.current = false;
          }
        })();
      },
      onFailed: (payload: CrawlerEventJobFailed) => {
        if (cancelled) {
          return;
        }
        let activeJobId = jobIdRef.current;
        if (!activeJobId) {
          jobIdRef.current = payload.job_id;
          setJobId(payload.job_id);
          activeJobId = payload.job_id;
        }
        if (payload.job_id !== activeJobId) {
          return;
        }
        setStatus("failed");
        setError(payload.message);
        setBusy(false);
        autoLoopRef.current = false;
        setAutoLoop(false);
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

  /**
   * 启动采集：优先走 AI 生成批次，否则用已选 batch。
   *
   * @author Xiaoman
   * @created 2026-07-23
   */
  async function start() {
    stopRequestedRef.current = false;
    autoLoopRef.current = autoLoop;
    aiLoopEnabledRef.current = aiReady;
    if (aiReady) {
      await generateAndStart();
      return;
    }
    await startCrawl();
  }

  async function cancel() {
    stopRequestedRef.current = true;
    autoLoopRef.current = false;
    aiLoopEnabledRef.current = false;
    setAutoLoop(false);
    loopInFlightRef.current = false;
    if (!jobId) {
      setBusy(false);
      setGeneratePhase((prev) => (prev === "running" ? "idle" : prev));
      return;
    }
    try {
      await crawlerJobCancel({ job_id: jobId });
    } catch (err) {
      setError(toDisplayError(err));
    }
  }

  /**
   * 由 Workflow Runtime 拉起 crawl 前，武装 UI 监听（不直接 start job）。
   *
   * @author Xiaoman
   * @created 2026-07-23
   */
  function armForRuntimeCrawl() {
    setError("");
    setBusy(true);
    setMessage(t("crawler.starting"));
    setStopReason("");
    setCurrentKeyword("");
    setKeywordStats([]);
    setLogs([]);
    setChannelResults([]);
    setAcceptedCount(0);
    setScannedCount(0);
    setKeywordsTotal(0);
    setKeywordsDone(0);
    jobIdRef.current = null;
    setJobId(null);
    // Runtime 负责编排；关闭前端 autoLoop 连跑，避免双通道。
    aiLoopEnabledRef.current = false;
    stopRequestedRef.current = false;
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
    directions,
    setDirections,
    languages,
    setLanguages,
    countPerLanguage,
    setCountPerLanguage,
    autoLoop,
    setAutoLoop,
    aiReady,
    generatePhase,
    generateMessage,
    lastGeneratedKeywords,
    jobId,
    status,
    statusText,
    stopReason,
    message,
    currentKeyword,
    acceptedCount,
    scannedCount,
    keywordsTotal,
    keywordsDone,
    keywordStats,
    logs,
    channelResults,
    busy,
    error,
    start,
    cancel,
    armForRuntimeCrawl,
  };
}
