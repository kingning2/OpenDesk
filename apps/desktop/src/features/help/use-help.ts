/**
 * Help hook — 一问一答的系统导航问答。
 *
 * 每次问答相互独立：不落库、无会话、无长期记忆、不带历史（后端 `help_ask`
 * 只注入系统操作指南 + 动作工具）。流式事件全局广播、按
 * `session_id === "help"` 且 `message_id` 匹配过滤；发送新问题时清空上一轮，
 * 重新开始一问一答。
 *
 * @author coisini
 */

import { useCallback, useEffect, useRef, useState } from "react";
import type { ChatEventToken, ChatEventTool } from "@desk/contracts";
import { listenChatEvents } from "@desk/platform/ipc/chat-events";
import { helpAsk } from "@desk/platform/ipc/help";
import { uuid } from "@desk/utils";

import { deriveAction, type ToolStep } from "../chat/chat-tool-utils";

/** 帮助页固定会话 id（与后端 `HELP_SESSION_ID` 对齐）。 */
export const HELP_SESSION_ID = "help";

/** 当前一轮问答的状态。 */
export interface HelpReply {
  /** 用户问题文本。 */
  question: string;
  /** 已累积的 AI 回复文本。 */
  content: string;
  /** 已累积的推理内容；模型不输出推理时为空。 */
  thinking: string;
  /** 动作工具（navigate_page / open_settings）解析出的步骤。 */
  tools: ToolStep[];
  /** 正在流式输出中。 */
  streaming: boolean;
  /** 该条回复出错。 */
  error?: boolean;
}

const toDisplayError = (err: unknown): string =>
  err instanceof Error ? err.message : String(err);

/**
 * 一问一答帮助问答 hook。
 *
 * @author coisini
 *
 * @returns 当前一轮回复、发送状态与动作
 */
export function useHelp() {
  const [reply, setReply] = useState<HelpReply | null>(null);
  const [sending, setSending] = useState(false);
  const [error, setError] = useState("");

  // 当前一轮回复的 message_id：事件按此过滤，避免串到上一轮或 Chat 页会话。
  const currentMessageIdRef = useRef("");
  // 回复的可变副本：token 事件里就地更新，避免闭包读到旧值覆盖新值。
  const replyRef = useRef<HelpReply | null>(null);

  const patchReply = useCallback((patch: Partial<HelpReply>) => {
    const current = replyRef.current;
    if (!current) {
      return;
    }
    const next = { ...current, ...patch };
    replyRef.current = next;
    setReply(next);
  }, []);

  const applyToken = useCallback(
    (payload: ChatEventToken) => {
      if (payload.session_id !== HELP_SESSION_ID) {
        return;
      }
      if (payload.message_id !== currentMessageIdRef.current) {
        return;
      }
      const current = replyRef.current;
      if (!current) {
        return;
      }
      const patch: Partial<HelpReply> = {};
      if (payload.reasoning) {
        patch.thinking = current.thinking + payload.reasoning;
      }
      if (payload.delta) {
        patch.content = current.content + payload.delta;
      }
      if (payload.done) {
        patch.streaming = false;
      }
      if (payload.error_message) {
        patch.error = true;
      }
      patchReply(patch);
    },
    [patchReply],
  );

  const applyTool = useCallback(
    (payload: ChatEventTool) => {
      if (payload.session_id !== HELP_SESSION_ID) {
        return;
      }
      if (payload.message_id !== currentMessageIdRef.current) {
        return;
      }
      const current = replyRef.current;
      if (!current) {
        return;
      }
      const step: ToolStep = {
        name: payload.name,
        arguments: payload.arguments,
        ok: payload.ok,
        result: payload.result,
      };
      const action = deriveAction(step);
      patchReply({
        tools: [...current.tools, action ? { ...step, action } : step],
      });
    },
    [patchReply],
  );

  // 挂载时订阅一次。StrictMode 双挂载用 disposed 标记确保只保留一个活跃订阅。
  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listenChatEvents({ onMessageToken: applyToken, onMessageTool: applyTool })
      .then((dispose) => {
        if (disposed) {
          dispose();
          return;
        }
        unlisten = dispose;
      })
      .catch((err) => {
        if (!disposed) {
          setError(toDisplayError(err));
        }
      });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [applyToken, applyTool]);

  const send = useCallback(
    async (text: string) => {
      const trimmed = text.trim();
      if (!trimmed || sending) {
        return;
      }
      const messageId = uuid();
      const entry: HelpReply = {
        question: trimmed,
        content: "",
        thinking: "",
        tools: [],
        streaming: true,
      };
      replyRef.current = entry;
      currentMessageIdRef.current = messageId;
      setReply(entry);
      setSending(true);
      setError("");
      try {
        await helpAsk({ text: trimmed, message_id: messageId });
        patchReply({ streaming: false });
      } catch (err) {
        patchReply({ streaming: false, error: true });
        setError(toDisplayError(err));
      } finally {
        setSending(false);
      }
    },
    [sending, patchReply],
  );

  const clearError = useCallback(() => setError(""), []);

  return { reply, sending, error, send, clearError };
}
