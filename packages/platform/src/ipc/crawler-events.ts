/**
 * Crawler Tauri event topics（枚举）与 typed listeners。
 *
 * Topic 与 Rust [`CrawlerUiEvent`] 一一对应；禁止散落字符串字面量。
 * Tauri 事件名只允许字母数字、`-`、`/`、`:`、`_`（禁止 `.`）。
 *
 * @author coisini
 * @created 2026-07-21
 */

import type {
  CrawlerEventChannelAccepted,
  CrawlerEventJobCompleted,
  CrawlerEventJobFailed,
  CrawlerEventJobLog,
  CrawlerEventJobProgress,
  CrawlerEventJobStarted,
} from "@desk/contracts";
import type { UnlistenFn } from "@tauri-apps/api/event";

import { listenEvent } from "../events";

/**
 * Crawler → UI Tauri event topics（与 Rust `CrawlerUiEvent` 对齐）。
 *
 * @author coisini
 * @created 2026-07-21
 */
export enum CrawlerUiEvent {
  /** Job entered running / keywords prepared. */
  JobStarted = "crawler:job/started",
  /** Progress snapshot (counts, message, keyword stats). */
  JobProgress = "crawler:job/progress",
  /** One process log line. */
  JobLog = "crawler:job/log",
  /** Job finished successfully or cancelled. */
  JobCompleted = "crawler:job/completed",
  /** Job failed with error. */
  JobFailed = "crawler:job/failed",
  /** One accepted channel row persisted. */
  ChannelAccepted = "crawler:channel/accepted",
  /** Email enrichment finished for one channel. */
  ChannelEmailEnriched = "crawler:channel/email_enriched",
}

/**
 * Subscribe to all crawler UI push events for one job session.
 *
 * Handlers ignore events whose `job_id` does not match (when provided by caller).
 *
 * @author coisini
 * @created 2026-07-21
 *
 * @param handlers - Per-topic callbacks
 * @returns Unlisten that tears down all subscriptions
 */
export async function listenCrawlerEvents(handlers: {
  onStarted?: (payload: CrawlerEventJobStarted) => void;
  onProgress?: (payload: CrawlerEventJobProgress) => void;
  onLog?: (payload: CrawlerEventJobLog) => void;
  onCompleted?: (payload: CrawlerEventJobCompleted) => void;
  onFailed?: (payload: CrawlerEventJobFailed) => void;
  onChannelAccepted?: (payload: CrawlerEventChannelAccepted) => void;
}): Promise<UnlistenFn> {
  const unlisteners = await Promise.all([
    handlers.onStarted
      ? listenEvent(CrawlerUiEvent.JobStarted, handlers.onStarted)
      : Promise.resolve(() => undefined),
    handlers.onProgress
      ? listenEvent(CrawlerUiEvent.JobProgress, handlers.onProgress)
      : Promise.resolve(() => undefined),
    handlers.onLog
      ? listenEvent(CrawlerUiEvent.JobLog, handlers.onLog)
      : Promise.resolve(() => undefined),
    handlers.onCompleted
      ? listenEvent(CrawlerUiEvent.JobCompleted, handlers.onCompleted)
      : Promise.resolve(() => undefined),
    handlers.onFailed
      ? listenEvent(CrawlerUiEvent.JobFailed, handlers.onFailed)
      : Promise.resolve(() => undefined),
    handlers.onChannelAccepted
      ? listenEvent(CrawlerUiEvent.ChannelAccepted, handlers.onChannelAccepted)
      : Promise.resolve(() => undefined),
  ]);

  return () => {
    for (const unlisten of unlisteners) {
      unlisten();
    }
  };
}
