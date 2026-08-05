/**
 * Chat hook — 会话内多轮（内存）+ 流式 token 逐字拼接。
 *
 * 历史不落库：每次发送时由 hook 把已完成的消息编码成 `messages_json`
 * 传给后端，后端追加本轮文本后流式回传 token。
 *
 * @author coisini
 */

import { useCallback, useEffect, useRef, useState } from "react";
import { chatSend } from "@desk/platform/ipc/chat";
import { listenChatEvents } from "@desk/platform/ipc/chat-events";
import type { ChatEventToken, ChatEventTool } from "@desk/contracts";

/** 一次工具调用（LLM 查询业务库时产生）。 */
export interface ToolStep {
  /** 工具名（如 `list_databases` / `run_query`）。 */
  name: string;
  /** JSON 编码的工具参数。 */
  arguments: string;
  /** 调用是否成功。 */
  ok: boolean;
  /** 工具返回（JSON 字符串）；失败时为错误描述。 */
  result?: string;
}

/** 一条聊天消息。 */
export interface ChatMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
  /** 已累积的推理内容（thinking）；模型不输出推理时为空。 */
  thinking?: string;
  /** 该回复期间执行的工具调用（按执行顺序）。 */
  tools?: ToolStep[];
  /** 正在流式输出中。 */
  streaming?: boolean;
  /** 该条回复出错。 */
  error?: boolean;
}

const toDisplayError = (err: unknown): string =>
  err instanceof Error ? err.message : String(err);

/**
 * 内存态多轮对话 hook。
 *
 * @author coisini
 *
 * @returns 消息列表、发送动作与错误状态
 */
export function useChat() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [sending, setSending] = useState(false);
  const [error, setError] = useState("");
  // 会话 id 一旦确定不再变化；用 ref 避免回调依赖震荡。
  const sessionIdRef = useRef(crypto.randomUUID());
  const sendingRef = useRef(false);
  // 按后端 message_id 累积增量：invoke 响应晚于首个 token，需缓冲以免丢失。
  const accumRef = useRef<
    Map<
      string,
      {
        content: string;
        thinking: string;
        tools: ToolStep[];
        done: boolean;
        error?: string;
      }
    >
  >(new Map());

  const emptyEntry = (): {
    content: string;
    thinking: string;
    tools: ToolStep[];
    done: boolean;
    error?: string;
  } => ({ content: "", thinking: "", tools: [], done: false });

  // 从累积区重建消息：token 与 tool 事件共用同一入口，保证顺序正确。
  const applyEntry = useCallback((messageId: string) => {
    const entry = accumRef.current.get(messageId);
    if (!entry) {
      return;
    }
    setMessages((prev) => {
      const next: ChatMessage = {
        id: messageId,
        role: "assistant",
        content: entry.content,
        thinking: entry.thinking || undefined,
        tools: entry.tools.length > 0 ? entry.tools : undefined,
        streaming: !entry.done,
        error: entry.error ? true : undefined,
      };
      const index = prev.findIndex((item) => item.id === messageId);
      if (index < 0) {
        return [...prev, next];
      }
      const copy = prev.slice();
      copy[index] = { ...copy[index], ...next };
      return copy;
    });
  }, []);

  const applyToken = useCallback(
    (payload: ChatEventToken) => {
      // 事件全局广播：只处理本会话的 token，忽略其它窗口/会话。
      if (payload.session_id !== sessionIdRef.current) {
        return;
      }
      const entry =
        accumRef.current.get(payload.message_id) ?? emptyEntry();
      if (payload.reasoning) {
        entry.thinking += payload.reasoning;
      }
      if (payload.delta) {
        entry.content += payload.delta;
      }
      if (payload.done) {
        entry.done = true;
      }
      if (payload.error_message) {
        entry.error = payload.error_message;
      }
      accumRef.current.set(payload.message_id, entry);
      applyEntry(payload.message_id);

      if (entry.done || entry.error) {
        sendingRef.current = false;
        setSending(false);
      }
    },
    [applyEntry],
  );

  const applyTool = useCallback(
    (payload: ChatEventTool) => {
      // 事件全局广播：只处理本会话的工具事件。
      if (payload.session_id !== sessionIdRef.current) {
        return;
      }
      const entry =
        accumRef.current.get(payload.message_id) ?? emptyEntry();
      entry.tools.push({
        name: payload.name,
        arguments: payload.arguments,
        ok: payload.ok,
        result: payload.result,
      });
      accumRef.current.set(payload.message_id, entry);
      applyEntry(payload.message_id);
    },
    [applyEntry],
  );

  // 挂载时订阅 token 事件一次。StrictMode 下 effect 会「挂载→清理→再挂载」跑两遍，
  // 用 effect 局部的 disposed 标记（而非共享 ref）确保每次只保留一个活跃订阅，
  // 迟到的旧订阅 promise 解析时立即自毁。
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

  const clearError = useCallback(() => setError(""), []);

  const send = useCallback(
    async (text: string) => {
      const trimmed = text.trim();
      if (!trimmed || sendingRef.current) {
        return;
      }
      sendingRef.current = true;
      setSending(true);
      setError("");

      const history = messages
        .filter((item) => !item.streaming && item.content.trim().length > 0)
        .map((item) => ({ role: item.role, content: item.content }));
      const userMessage: ChatMessage = {
        id: crypto.randomUUID(),
        role: "user",
        content: trimmed,
      };
      // 立即追加 assistant 占位消息，首个 token 前的空窗也有「思考中」反馈；
      // id 随请求传给后端，使 token 事件落到这条占位上。
      const assistantId = crypto.randomUUID();
      const assistantPlaceholder: ChatMessage = {
        id: assistantId,
        role: "assistant",
        content: "",
        streaming: true,
      };
      setMessages((prev) => [...prev, userMessage, assistantPlaceholder]);

      try {
        await chatSend({
          session_id: sessionIdRef.current,
          messages_json: JSON.stringify(history),
          text: trimmed,
          message_id: assistantId,
        });
      } catch (err) {
        // 首个 token 前失败（未配置 / 网络错误）：撤掉占位，交给错误条提示。
        setMessages((prev) =>
          prev.filter((item) => item.id !== assistantId),
        );
        setError(toDisplayError(err));
      } finally {
        sendingRef.current = false;
        setSending(false);
      }
    },
    [messages],
  );

  return { messages, sending, error, clearError, send };
}
