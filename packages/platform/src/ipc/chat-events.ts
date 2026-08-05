/**
 * Chat Tauri event topics（枚举）与 typed listeners。
 *
 * Topic 与 Rust [`ChatUiEvent`] 一一对应；禁止散落字符串字面量。
 * Tauri 事件名只允许字母数字、`-`、`/`、`:`、`_`（禁止 `.`）。
 *
 * @author coisini
 */

import type { ChatEventToken, ChatEventTool } from "@desk/contracts";
import type { UnlistenFn } from "@tauri-apps/api/event";

import { listenEvent } from "../events";

/**
 * Chat → UI Tauri event topics（与 Rust `ChatUiEvent` 对齐）。
 *
 * @author coisini
 */
export enum ChatUiEvent {
  /** One streamed token (or the final done event) of an assistant reply. */
  MessageToken = "chat:message/token",
  /** One tool call executed by the assistant during a reply. */
  MessageTool = "chat:message/tool",
}

/**
 * Subscribe to chat streaming events.
 *
 * @author coisini
 *
 * @param handlers - Per-topic callbacks
 * @returns Unlisten that tears down all subscriptions
 */
export async function listenChatEvents(handlers: {
  onMessageToken?: (payload: ChatEventToken) => void;
  onMessageTool?: (payload: ChatEventTool) => void;
}): Promise<UnlistenFn> {
  const unlisteners = await Promise.all([
    handlers.onMessageToken
      ? listenEvent(ChatUiEvent.MessageToken, handlers.onMessageToken)
      : Promise.resolve(() => undefined),
    handlers.onMessageTool
      ? listenEvent(ChatUiEvent.MessageTool, handlers.onMessageTool)
      : Promise.resolve(() => undefined),
  ]);

  return () => {
    for (const unlisten of unlisteners) {
      unlisten();
    }
  };
}
