/**
 * Crawl job hook — start/cancel/status + process log polling.
 *
 * @author Xiaoman
 * @created 2026-07-16
 */

import { useEffect, useState } from "react";
import {
  crawlerJobCancel,
  crawlerJobLogs,
  crawlerJobStart,
  crawlerJobStatus,
} from "@desk/platform/ipc/crawler";

export interface CrawlLogLine {
  seq?: number;
  phase?: string;
  level?: string;
  message?: string;
  keyword?: string;
}

export function useCrawlerJob() {
  const [apiKey, setApiKey] = useState("");
  const [keywords, setKeywords] = useState("beauty,skincare");
  const [jobId, setJobId] = useState<string | null>(null);
  const [status, setStatus] = useState("idle");
  const [summary, setSummary] = useState("");
  const [logs, setLogs] = useState<CrawlLogLine[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    if (!jobId) {
      return;
    }
    let cancelled = false;

    async function poll() {
      try {
        const [statusRes, logsRes] = await Promise.all([
          crawlerJobStatus({ job_id: jobId! }),
          crawlerJobLogs({ job_id: jobId! }),
        ]);
        if (cancelled) {
          return;
        }
        setStatus(statusRes.status);
        setSummary(
          [
            `platform=${statusRes.platform}`,
            statusRes.scanned_count != null ? `scanned=${statusRes.scanned_count}` : null,
            statusRes.accepted_count != null ? `accepted=${statusRes.accepted_count}` : null,
            statusRes.stop_reason ? `stop=${statusRes.stop_reason}` : null,
          ]
            .filter(Boolean)
            .join(" · "),
        );
        try {
          const parsed = JSON.parse(logsRes.logs_json) as CrawlLogLine[];
          setLogs(Array.isArray(parsed) ? parsed : []);
        } catch {
          setLogs([]);
        }
        if (statusRes.status === "completed" || statusRes.status === "failed" || statusRes.status === "cancelled") {
          setBusy(false);
        }
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : String(err));
          setBusy(false);
        }
      }
    }

    void poll();
    const timer = window.setInterval(() => {
      void poll();
    }, 1000);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, [jobId]);

  async function start() {
    setError("");
    setBusy(true);
    setLogs([]);
    setSummary("");
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
        return;
      }
      setJobId(result.job_id);
      setStatus("queued");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setBusy(false);
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
    summary,
    logs,
    busy,
    error,
    start,
    cancel,
  };
}
