/**
 * 渠道 store — 账号/会话/消息/设置，整体持久化到 Rust 端。
 */

import type {
  ChannelAccount,
  ChannelConversation,
  ChannelMessage,
  ChannelSettings,
} from "@desk/contracts";
import { channelStateGet, channelStateSet } from "@desk/platform/ipc/channel";
import { createDeskStore } from "@desk/store";

export interface ChannelState {
  /** 渠道账号列表。 */
  accounts: ChannelAccount[];
  /** 会话列表。 */
  conversations: ChannelConversation[];
  /** 全部消息（按会话筛选展示）。 */
  messages: ChannelMessage[];
  /** 设置。 */
  settings: ChannelSettings;
  /** 是否加载中。 */
  loading: boolean;
  /** 是否已加载。 */
  loaded: boolean;
  /** 最近错误。 */
  error: string | null;
  /** 当前选中会话 id。 */
  activeConversationId: string | null;
  /** 加载全量状态。 */
  load: () => Promise<void>;
  /** 保存账号列表 + 设置（整体持久化）。 */
  save: (accounts: ChannelAccount[], settings: ChannelSettings) => Promise<void>;
  /** 新增/更新账号。 */
  upsertAccount: (account: ChannelAccount) => Promise<void>;
  /** 删除账号。 */
  removeAccount: (id: string) => Promise<void>;
  /** 切换自动回复。 */
  setAutoReply: (enabled: boolean) => Promise<void>;
  /** 选择会话。 */
  selectConversation: (id: string | null) => void;
  /** 追加一条消息（入站/出站事件推送）。 */
  appendMessage: (message: ChannelMessage) => void;
}

function toError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export const useChannelStore = createDeskStore<ChannelState>((set, get) => ({
  accounts: [],
  conversations: [],
  messages: [],
  settings: { auto_reply: false },
  loading: false,
  loaded: false,
  error: null,
  activeConversationId: null,

  load: async () => {
    set({ loading: true, error: null });
    try {
      const state = await channelStateGet();
      set({
        accounts: state.accounts,
        conversations: state.conversations,
        messages: state.messages,
        settings: state.settings,
        loading: false,
        loaded: true,
      });
    } catch (error) {
      set({ error: toError(error), loading: false });
    }
  },

  save: async (accounts, settings) => {
    try {
      const saved = await channelStateSet({ accounts, settings });
      set({ accounts: saved.accounts, settings: saved.settings });
    } catch (error) {
      set({ error: toError(error) });
      throw error;
    }
  },

  upsertAccount: async (account) => {
    const { accounts, settings } = get();
    const existing = accounts.some((item) => item.id === account.id);
    await get().save(
      existing
        ? accounts.map((item) => (item.id === account.id ? account : item))
        : [...accounts, account],
      settings,
    );
  },

  removeAccount: async (id) => {
    const { accounts, settings } = get();
    await get().save(
      accounts.filter((item) => item.id !== id),
      settings,
    );
  },

  setAutoReply: async (enabled) => {
    const { accounts } = get();
    await get().save(accounts, { auto_reply: enabled });
  },

  selectConversation: (id) => set({ activeConversationId: id }),

  appendMessage: (message) => {
    const { messages } = get();
    // 幂等：同 id 不重复追加。
    if (messages.some((item) => item.id === message.id)) {
      return;
    }
    set({ messages: [...messages, message] });
  },
}));
