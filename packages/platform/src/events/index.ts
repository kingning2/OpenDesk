/**
 * 事件订阅封装：桌面走 Tauri `listen`，浏览器 / web 端走 SSE `EventSource`。
 *
 * @author coisini
 * @created 2026-07-21
 * @updated 2026-08-08
 */

import type { UnlistenFn } from "@tauri-apps/api/event";

import { isTauriRuntime } from "../ipc/invoke";

/** 单例 SSE EventSource（多个 topic 共享一个连接）。 */
let sharedSource: EventSource | null = null;
let sharedHandlerCount = 0;
const sseHandlers = new Map<string, Set<(payload: unknown) => void>>();

function ensureSource(base: string): void {
  if (sharedSource) {
    return;
  }
  const source = new EventSource(`${base}/api/events`);
  // SSE 命名事件：`event: <topic>` 通过 addEventListener 接收。
  source.addEventListener("crawler:job/started", (event) => dispatch("crawler:job/started", event));
  source.addEventListener("crawler:job/progress", (event) => dispatch("crawler:job/progress", event));
  source.addEventListener("crawler:job/log", (event) => dispatch("crawler:job/log", event));
  source.addEventListener("crawler:job/completed", (event) => dispatch("crawler:job/completed", event));
  source.addEventListener("crawler:job/failed", (event) => dispatch("crawler:job/failed", event));
  source.addEventListener("crawler:channel/accepted", (event) => dispatch("crawler:channel/accepted", event));
  source.addEventListener("crawler:channel/email_enriched", (event) => dispatch("crawler:channel/email_enriched", event));
  source.addEventListener("chat:message/token", (event) => dispatch("chat:message/token", event));
  source.addEventListener("chat:message/tool", (event) => dispatch("chat:message/tool", event));
  source.addEventListener("mail:imap-sync-updated", (event) => dispatch("mail:imap-sync-updated", event));
  source.addEventListener("knowledge:import/updated", (event) => dispatch("knowledge:import/updated", event));
  source.addEventListener("knowledge:tool/progress", (event) => dispatch("knowledge:tool/progress", event));
  source.addEventListener("workflow_runtime:phase", (event) => dispatch("workflow_runtime:phase", event));
  sharedSource = source;
}

/** 把 SSE 命名事件分发给对应 topic 的 handler。 */
function dispatch(topic: string, event: MessageEvent): void {
  const handlers = sseHandlers.get(topic);
  handlers?.forEach((handler) => {
    try {
      handler(JSON.parse(event.data as string));
    } catch {
      // ignore malformed payload
    }
  });
}

/**
 * 订阅一个事件 topic。
 *
 * @typeParam T - Payload 类型
 * @param topic - 事件 topic，如 `crawler:job/progress`
 * @param handler - 回调
 * @returns 取消订阅函数
 */
export async function listenEvent<T>(
  topic: string,
  handler: (payload: T) => void,
): Promise<UnlistenFn> {
  if (!isTauriRuntime()) {
    const base = (typeof window !== "undefined" && (window as { __OPENDESK_SERVER__?: string }).__OPENDESK_SERVER__) || "";
    ensureSource(base);
    if (!sseHandlers.has(topic)) {
      sseHandlers.set(topic, new Set());
    }
    sseHandlers.get(topic)!.add(handler as (payload: unknown) => void);
    sharedHandlerCount += 1;

    return () => {
      const set = sseHandlers.get(topic);
      set?.delete(handler as (payload: unknown) => void);
      sharedHandlerCount -= 1;
      if (sharedHandlerCount <= 0 && sharedSource) {
        sharedSource.close();
        sharedSource = null;
        sseHandlers.clear();
        sharedHandlerCount = 0;
      }
    };
  }

  const { listen } = await import("@tauri-apps/api/event");
  return listen<T>(topic, (event) => {
    handler(event.payload);
  });
}
