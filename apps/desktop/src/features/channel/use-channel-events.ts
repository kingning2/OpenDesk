/**
 * 渠道事件订阅 — 监听 Rust 端推送的入站/出站消息与连接状态，增量更新 store。
 */

import { useEffect } from "react";
import type { ChannelEventMessage } from "@desk/contracts";
import {
  listenChannelMessage,
  listenChannelStatus,
} from "@desk/platform/events/channel";
import { useChannelStore } from "./use-channel-store";

/**
 * 订阅渠道事件。挂载时监听，卸载时取消。
 *
 * 消息事件：追加到 store.messages（含 AI 建议，展示在输入区）。
 * 状态事件：可扩展连接状态展示（本次 store 未持久化状态，预留）。
 */
export function useChannelEvents() {
  useEffect(() => {
    let cancelled = false;
    let unlistenMessage: (() => void) | undefined;
    let unlistenStatus: (() => void) | undefined;

    void listenChannelMessage((event: ChannelEventMessage) => {
      if (cancelled) {
        return;
      }
      const { appendMessage } = useChannelStore.getState();
      appendMessage(event.message);
      // 建议文案暂存（后续可挂在输入区）。
      if (event.suggestion) {
        window.dispatchEvent(
          new CustomEvent("channel:suggestion", { detail: event.suggestion }),
        );
      }
    }).then((unlisten) => {
      if (cancelled) {
        unlisten();
      } else {
        unlistenMessage = unlisten;
      }
    });

    void listenChannelStatus(() => {
      // 连接状态展示（后续扩展）。
    }).then((unlisten) => {
      if (cancelled) {
        unlisten();
      } else {
        unlistenStatus = unlisten;
      }
    });

    return () => {
      cancelled = true;
      unlistenMessage?.();
      unlistenStatus?.();
    };
  }, []);
}
