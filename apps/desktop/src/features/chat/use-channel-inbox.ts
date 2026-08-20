/**
 * 渠道收件箱状态 — 读取 SQLite 会话/消息并订阅实时推送。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */

import { useCallback, useEffect, useMemo, useState } from "react";
import type {
  ChannelConversation,
  ChannelEventMessage,
  ChannelMessage,
} from "@desk/contracts";
import { listenChannelMessage } from "@desk/platform/events";
import { channelSend, channelStateGet } from "@desk/platform/ipc/channel";

/** 收件箱聚合状态。 */
export interface ChannelInboxState {
  /** 是否首次加载中。 */
  loading: boolean;
  /** 加载或发送错误。 */
  error: string | null;
  /** 全部会话。 */
  conversations: ChannelConversation[];
  /** 全部消息。 */
  messages: ChannelMessage[];
  /** 当前选中会话 id。 */
  selectedId: string | null;
  /** 选中会话。 */
  selectedConversation: ChannelConversation | null;
  /** 当前会话消息（按时间升序）。 */
  threadMessages: ChannelMessage[];
  /** 刷新渠道状态。 */
  refresh: () => Promise<void>;
  /** 选中会话。 */
  selectConversation: (conversationId: string) => void;
  /** 发送文本消息。 */
  sendMessage: (content: string) => Promise<void>;
  /** 按账号筛选（空 = 全部）。 */
  accountFilter: string;
  /** 设置账号筛选。 */
  setAccountFilter: (accountId: string) => void;
  /** 筛选后的会话列表（按 updated_at 降序）。 */
  filteredConversations: ChannelConversation[];
}

/**
 * 管理渠道收件箱：初始拉取 + 实时事件合并。
 *
 * @author Xiaoman
 * @created 2026-08-20
 *
 * @returns 收件箱状态与操作
 */
export function useChannelInbox(): ChannelInboxState {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [conversations, setConversations] = useState<ChannelConversation[]>([]);
  const [messages, setMessages] = useState<ChannelMessage[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [accountFilter, setAccountFilter] = useState("");

  const refresh = useCallback(async () => {
    setError(null);
    try {
      const state = await channelStateGet();
      setConversations(state.conversations);
      setMessages(state.messages);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    void listenChannelMessage((payload: ChannelEventMessage) => {
      if (cancelled) {
        return;
      }
      const { message } = payload;
      setMessages((current) => {
        if (current.some((item) => item.id === message.id)) {
          return current;
        }
        return [...current, message];
      });
      setConversations((current) => {
        const existing = current.find((item) => item.id === message.conversation_id);
        const updated: ChannelConversation = existing
          ? {
              ...existing,
              updated_at: message.created_at,
            }
          : {
              id: message.conversation_id,
              account_id: payload.account_id,
              peer_id: "",
              peer_name: "未知联系人",
              item_id: "",
              updated_at: message.created_at,
            };
        const rest = current.filter((item) => item.id !== message.conversation_id);
        return [updated, ...rest];
      });
    }).then((dispose) => {
      if (cancelled) {
        dispose();
        return;
      }
      unlisten = dispose;
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  const filteredConversations = useMemo(() => {
    const list = accountFilter
      ? conversations.filter((item) => item.account_id === accountFilter)
      : conversations;
    return [...list].sort((a, b) => Number(b.updated_at) - Number(a.updated_at));
  }, [accountFilter, conversations]);

  const selectedConversation = useMemo(
    () => conversations.find((item) => item.id === selectedId) ?? null,
    [conversations, selectedId],
  );

  const threadMessages = useMemo(() => {
    if (!selectedId) {
      return [];
    }
    return messages
      .filter((item) => item.conversation_id === selectedId)
      .sort((a, b) => Number(a.created_at) - Number(b.created_at));
  }, [messages, selectedId]);

  const sendMessage = useCallback(
    async (content: string) => {
      const trimmed = content.trim();
      if (!selectedId || !trimmed) {
        return;
      }
      setError(null);
      await channelSend({
        conversation_id: selectedId,
        content: trimmed,
      });
      await refresh();
    },
    [refresh, selectedId],
  );

  return {
    loading,
    error,
    conversations,
    messages,
    selectedId,
    selectedConversation,
    threadMessages,
    refresh,
    selectConversation: setSelectedId,
    sendMessage,
    accountFilter,
    setAccountFilter,
    filteredConversations,
  };
}

/**
 * 取会话最后一条消息摘要。
 *
 * @author Xiaoman
 * @created 2026-08-20
 *
 * @param conversationId - 会话 id
 * @param messages - 全部消息
 * @returns 摘要文本；无消息时返回空串
 */
export function lastMessagePreview(
  conversationId: string,
  messages: ChannelMessage[],
): string {
  const thread = messages
    .filter((item) => item.conversation_id === conversationId)
    .sort((a, b) => Number(b.created_at) - Number(a.created_at));
  return thread[0]?.content ?? "";
}

/**
 * 会话是否有待回复入站消息（最后一条为 inbound）。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export function conversationNeedsReply(
  conversationId: string,
  messages: ChannelMessage[],
): boolean {
  const thread = messages
    .filter((item) => item.conversation_id === conversationId)
    .sort((a, b) => Number(b.created_at) - Number(a.created_at));
  return thread[0]?.direction === "in";
}
