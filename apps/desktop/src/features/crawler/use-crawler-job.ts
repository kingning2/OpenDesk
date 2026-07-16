/**
 * Crawl job hook — poll operational progress (keyword / counts / stop reason).
 *
 * @author Xiaoman
 * @created 2026-07-16
 */

import { useEffect, useState } from "react";
import {
  crawlerJobCancel,
  crawlerJobStart,
  crawlerJobStatus,
} from "@desk/platform/ipc/crawler";

export interface KeywordStatRow {
  keyword: string;
  scanned: number;
  accepted: number;
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
  const [keywords, setKeywords] = useState("beauty,skincare");
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
  const [keywordStats, setKeywordStats] = useState<KeywordStatRow[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    if (!jobId) {
      return;
    }
    let cancelled = false;

    async function poll() {
      try {
        const statusRes = await crawlerJobStatus({ job_id: jobId! });
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
        if (statusRes.error_message) {
          setError(statusRes.error_message);
        }
        try {
          const parsed = JSON.parse(statusRes.keyword_stats_json ?? "[]") as KeywordStatRow[];
          setKeywordStats(Array.isArray(parsed) ? parsed : []);
        } catch {
          setKeywordStats([]);
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

  async function start() {
    setError("");
    setBusy(true);
    setMessage("正在启动…");
    setStopReason("");
    setCurrentKeyword("");
    setKeywordStats([]);
    setAcceptedCount(0);
    setScannedCount(0);
    setKeywordAccepted(0);
    setKeywordScanned(0);
    setQuotaUsed(0);
    try {
      const result = await crawlerJobStart({
        platform: "youtube",
        keywords,
        api_key: apiKey,
        max_total: 20,
        rate_limit_ms: 400,
        trace_id: crypto.randomUUID(),
      });
      if (!result.ok || !result.job_id) {
        setError("启动失败：请检查 API Key 与 Sidecar");
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

  return {
    apiKey,
    setApiKey,
    keywords,
    setKeywords,
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
    keywordStats,
    busy,
    error,
    start,
    cancel,
  };
}
