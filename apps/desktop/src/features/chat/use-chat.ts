/**
 * Chat hook — 多会话持久化 + 流式 token 逐字拼接。
 *
 * 历史由后端从 `chat.db` 重建，前端只传 `session_id + text + message_id`；
 * 切换会话时从 store 重新加载消息，流式事件全局广播、按 `session_id` 过滤，
 * 切走的会话仍在后台流式并落库，切回时自动合并进行中的回复。
 *
 * @author coisini
 */

import { useCallback, useEffect, useRef, useState } from "react";
import {
  chatMessagesLoad,
  chatSend,
  chatSessionCreate,
  chatSessionDelete,
  chatSessionList,
  chatSessionRename,
  type ChatMessage,
  type ChatSession,
} from "@desk/platform/ipc/chat";
import { listenChatEvents } from "@desk/platform/ipc/chat-events";
import type { ChatEventToken, ChatEventTool } from "@desk/contracts";

import { deriveAction, type ToolStep } from "./chat-tool-utils";

/** 一条聊天消息。 */
export interface ChatMessageView {
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

/** 流式累积区里一条 assistant 回复的状态。 */
interface StreamEntry {
  content: string;
  thinking: string;
  tools: ToolStep[];
  done: boolean;
  error?: string;
}

const toDisplayError = (err: unknown): string =>
  err instanceof Error ? err.message : String(err);

/** 解析后端落库的 `tools_json`（`{name,arguments,ok,result}[]` 字符串）。 */
function parseToolsJson(json?: string): ToolStep[] {
  if (!json) {
    return [];
  }
  try {
    const parsed = JSON.parse(json);
    return Array.isArray(parsed) ? (parsed as ToolStep[]) : [];
  } catch {
    return [];
  }
}

/** 把一条落库消息转成展示消息。 */
function fromPersistedMessage(dto: ChatMessage): ChatMessageView {
  const tools = parseToolsJson(dto.tools_json).map((step) => {
    const action = deriveAction(step);
    return action ? { ...step, action } : step;
  });
  return {
    id: dto.id,
    role: dto.role === "assistant" ? "assistant" : "user",
    content: dto.content,
    thinking: dto.thinking || undefined,
    tools: tools.length > 0 ? tools : undefined,
  };
}

/** 模块级：确保至少存在一个会话（首次启动 / 已删光全部会话时）。
 *  StrictMode 双挂载共享同一个 promise，避免重复创建；完成后立即重置，
 *  使「删光会话后再挂载」能重新创建。 */
let ensureSessionPromise: Promise<ChatSession> | null = null;

function ensureSession(): Promise<ChatSession> {
  if (!ensureSessionPromise) {
    ensureSessionPromise = chatSessionCreate({}).finally(() => {
      ensureSessionPromise = null;
    });
  }
  return ensureSessionPromise;
}

/**
 * 多会话持久化聊天 hook。
 *
 * @author coisini
 *
 * @returns 会话列表、当前会话消息与各类会话动作
 */
export function useChat() {
  const [sessions, setSessions] = useState<ChatSession[]>([]);
  const [activeSessionId, setActiveSessionId] = useState("");
  const [messages, setMessages] = useState<ChatMessageView[]>([]);
  const [sending, setSending] = useState(false);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);

  // 当前激活会话 id 的 ref（事件回调里读取最新值，避免闭包陈旧）。
  const activeSessionIdRef = useRef("");
  // 正在发送中的会话集合（后端仍在流式返回，send 按钮按会话禁用）。
  const inflightRef = useRef<Set<string>>(new Set());
  // 流式累积区：session_id → message_id → 状态；切走的会话也继续累积，切回时合并。
  const streamingBySessionRef = useRef<Map<string, Map<string, StreamEntry>>>(new Map());

  const emptyEntry = (): StreamEntry => ({
    content: "",
    thinking: "",
    tools: [],
    done: false,
  });

  const upsertStreamEntry = useCallback((sessionId: string, messageId: string) => {
    let sessionMap = streamingBySessionRef.current.get(sessionId);
    if (!sessionMap) {
      sessionMap = new Map();
      streamingBySessionRef.current.set(sessionId, sessionMap);
    }
    const entry = sessionMap.get(messageId) ?? emptyEntry();
    sessionMap.set(messageId, entry);
    return entry;
  }, []);

  // 从累积区重建当前会话的消息：token 与 tool 事件共用同一入口，保证顺序正确。
  const applyEntry = useCallback((sessionId: string, messageId: string) => {
    if (sessionId !== activeSessionIdRef.current) {
      return;
    }
    const sessionMap = streamingBySessionRef.current.get(sessionId);
    const entry = sessionMap?.get(messageId);
    if (!entry) {
      return;
    }
    const next: ChatMessageView = {
      id: messageId,
      role: "assistant",
      content: entry.content,
      thinking: entry.thinking || undefined,
      tools: entry.tools.length > 0 ? entry.tools : undefined,
      streaming: !entry.done,
      error: entry.error ? true : undefined,
    };
    setMessages((prev) => {
      const index = prev.findIndex((item) => item.id === messageId);
      if (index < 0) {
        return [...prev, next];
      }
      const copy = prev.slice();
      copy[index] = { ...copy[index], ...next };
      return copy;
    });
    // 已完成：消息已落在 messages 里，清理累积区避免陈旧覆盖 / 泄漏。
    if (entry.done) {
      sessionMap?.delete(messageId);
    }
  }, []);

  const applyToken = useCallback(
    (payload: ChatEventToken) => {
      // 事件全局广播：任何会话都累积，但只把激活会话的进度应用到界面。
      const entry = upsertStreamEntry(payload.session_id, payload.message_id);
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
      if (payload.session_id === activeSessionIdRef.current) {
        applyEntry(payload.session_id, payload.message_id);
      }
      if (entry.done || entry.error) {
        inflightRef.current.delete(payload.session_id);
        if (payload.session_id === activeSessionIdRef.current) {
          setSending(inflightRef.current.has(payload.session_id));
        }
      }
    },
    [applyEntry, upsertStreamEntry],
  );

  const applyTool = useCallback(
    (payload: ChatEventTool) => {
      const entry = upsertStreamEntry(payload.session_id, payload.message_id);
      const step: ToolStep = {
        name: payload.name,
        arguments: payload.arguments,
        ok: payload.ok,
        result: payload.result,
      };
      // 动作工具（navigate_page / open_settings）解析出可执行动作，交给 UI 渲染按钮；
      // 不自动跳转，由用户点击触发（数据查询类工具无 action，保持普通步骤展示）。
      const action = deriveAction(step);
      entry.tools.push(action ? { ...step, action } : step);
      if (payload.session_id === activeSessionIdRef.current) {
        applyEntry(payload.session_id, payload.message_id);
      }
    },
    [applyEntry, upsertStreamEntry],
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

  const refreshSessions = useCallback(async (): Promise<ChatSession[]> => {
    const items = await chatSessionList();
    setSessions(items);
    return items;
  }, []);

  // 加载某会话的已落库消息，并把该会话进行中的流式回复合并回列表。
  const loadSession = useCallback(async (sessionId: string) => {
    const dtos = await chatMessagesLoad({ session_id: sessionId });
    const persisted = dtos.map(fromPersistedMessage);
    const persistedIds = new Set(persisted.map((item) => item.id));
    const sessionMap = streamingBySessionRef.current.get(sessionId);
    const merged: ChatMessageView[] = [];
    if (sessionMap) {
      for (const [messageId, entry] of sessionMap.entries()) {
        if (entry.done || persistedIds.has(messageId)) {
          // 已完成或后端已落库（`chatSend` 已返回时后端必已写完）：
          // 由 persisted 提供，清理累积区避免陈旧覆盖 / 重复。
          sessionMap.delete(messageId);
        } else {
          merged.push({
            id: messageId,
            role: "assistant",
            content: entry.content,
            thinking: entry.thinking || undefined,
            tools: entry.tools.length > 0 ? entry.tools : undefined,
            streaming: true,
          });
        }
      }
    }
    setMessages([...persisted, ...merged]);
  }, []);

  // 挂载时初始化：加载会话列表（无会话则创建默认会话），再加载首个会话消息。
  useEffect(() => {
    let disposed = false;
    void (async () => {
      try {
        let items = await chatSessionList();
        if (items.length === 0) {
          await ensureSession();
          // 重新查询，避免被本地残留的空数组覆盖真实会话列表。
          items = await chatSessionList();
        }
        if (disposed) {
          return;
        }
        setSessions(items);
        const active = items[0]?.id ?? "";
        activeSessionIdRef.current = active;
        setActiveSessionId(active);
        if (active) {
          const dtos = await chatMessagesLoad({ session_id: active });
          if (!disposed) {
            setMessages(dtos.map(fromPersistedMessage));
          }
        }
      } catch (err) {
        if (!disposed) {
          setError(toDisplayError(err));
        }
      } finally {
        if (!disposed) {
          setLoading(false);
        }
      }
    })();
    return () => {
      disposed = true;
    };
  }, []);

  const clearError = useCallback(() => setError(""), []);

  const switchSession = useCallback(
    async (sessionId: string) => {
      if (sessionId === activeSessionIdRef.current) {
        return;
      }
      activeSessionIdRef.current = sessionId;
      setActiveSessionId(sessionId);
      setMessages([]);
      setError("");
      setSending(inflightRef.current.has(sessionId));
      try {
        await loadSession(sessionId);
      } catch (err) {
        setError(toDisplayError(err));
      }
    },
    [loadSession],
  );

  const createSession = useCallback(async (): Promise<ChatSession | null> => {
    try {
      const session = await chatSessionCreate({});
      setSessions((prev) => [session, ...prev]);
      activeSessionIdRef.current = session.id;
      setActiveSessionId(session.id);
      setMessages([]);
      setError("");
      setSending(false);
      return session;
    } catch (err) {
      setError(toDisplayError(err));
      return null;
    }
  }, []);

  const renameSession = useCallback(
    async (id: string, title: string): Promise<ChatSession | null> => {
      try {
        const session = await chatSessionRename({ id, title: title.trim() });
        setSessions((prev) => prev.map((item) => (item.id === id ? session : item)));
        return session;
      } catch (err) {
        setError(toDisplayError(err));
        return null;
      }
    },
    [],
  );

  const deleteSession = useCallback(
    async (id: string): Promise<boolean> => {
      try {
        await chatSessionDelete({ id });
        streamingBySessionRef.current.delete(id);
        inflightRef.current.delete(id);
        const remaining = sessions.filter((item) => item.id !== id);
        setSessions(remaining);
        if (activeSessionIdRef.current === id) {
          const next = remaining[0];
          if (next) {
            activeSessionIdRef.current = next.id;
            setActiveSessionId(next.id);
            setMessages([]);
            setSending(inflightRef.current.has(next.id));
            await loadSession(next.id);
          } else {
            activeSessionIdRef.current = "";
            setActiveSessionId("");
            setMessages([]);
            setSending(false);
          }
        }
        return true;
      } catch (err) {
        setError(toDisplayError(err));
        return false;
      }
    },
    [sessions, loadSession],
  );

  const send = useCallback(
    async (text: string) => {
      const trimmed = text.trim();
      if (!trimmed) {
        return;
      }
      let sessionId = activeSessionIdRef.current;
      if (!sessionId) {
        // 无活动会话（首次引导失败 / 已删光全部会话）：先自动创建一个再发送。
        const created = await createSession();
        if (!created) {
          return; // createSession 已 setError
        }
        sessionId = created.id;
      }
      if (inflightRef.current.has(sessionId)) {
        return;
      }
      inflightRef.current.add(sessionId);
      setSending(true);
      setError("");

      const userMessage: ChatMessageView = {
        id: crypto.randomUUID(),
        role: "user",
        content: trimmed,
      };
      // 立即追加 assistant 占位消息，首个 token 前的空窗也有「思考中」反馈；
      // id 随请求传给后端，使 token 事件落到这条占位上。
      const assistantId = crypto.randomUUID();
      const assistantPlaceholder: ChatMessageView = {
        id: assistantId,
        role: "assistant",
        content: "",
        streaming: true,
      };
      setMessages((prev) => [...prev, userMessage, assistantPlaceholder]);

      try {
        await chatSend({
          session_id: sessionId,
          text: trimmed,
          message_id: assistantId,
        });
        // 流式结束：刷新会话列表（首条消息后自动命名），并从 store 重载以对齐落库结果。
        const entry = streamingBySessionRef.current.get(sessionId)?.get(assistantId);
        await refreshSessions();
        if (!entry?.error && activeSessionIdRef.current === sessionId) {
          await loadSession(sessionId);
        }
      } catch (err) {
        // 首个 token 前失败（未配置 / 网络错误）：撤掉占位，交给错误条提示。
        setMessages((prev) => prev.filter((item) => item.id !== assistantId));
        setError(toDisplayError(err));
      } finally {
        inflightRef.current.delete(sessionId);
        setSending(inflightRef.current.has(activeSessionIdRef.current));
      }
    },
    [createSession, loadSession, refreshSessions],
  );

  return {
    sessions,
    activeSessionId,
    messages,
    sending,
    error,
    loading,
    clearError,
    send,
    switchSession,
    createSession,
    renameSession,
    deleteSession,
  };
}
