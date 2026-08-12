/**
 * 日志面板 store — 轮询 Rust 端日志缓冲，支持级别过滤。
 */

import { logClear, type LogEntry } from "@desk/platform/ipc/log";
import { createDeskStore } from "@desk/store";

export type LogLevelFilter = "all" | "debug" | "info" | "warn" | "error";

export interface LogState {
  /** 面板是否可见。 */
  visible: boolean;
  /** 已加载日志（时间正序）。 */
  entries: LogEntry[];
  /** 级别过滤（all 显示全部）。 */
  levelFilter: LogLevelFilter;
  /** 打开面板。 */
  open: () => void;
  /** 关闭面板。 */
  close: () => void;
  /** 切换面板。 */
  toggle: () => void;
  setLevelFilter: (filter: LogLevelFilter) => void;
  /** 追加新日志（轮询去重）。 */
  append: (entries: LogEntry[]) => void;
  /** 清空后端缓冲与本地展示。 */
  clear: () => Promise<void>;
}

const MAX_ENTRIES = 1000;

function entryKey(entry: LogEntry): string {
  return `${entry.ts}|${entry.level}|${entry.source}|${entry.message}`;
}

export const useLogStore = createDeskStore<LogState>((set, get) => ({
  visible: false,
  entries: [],
  levelFilter: "all",

  open: () => set({ visible: true }),
  close: () => set({ visible: false }),
  toggle: () => set((state) => ({ visible: !state.visible })),

  setLevelFilter: (filter) => set({ levelFilter: filter }),

  append: (entries) => {
    if (entries.length === 0) {
      return;
    }
    const { entries: existing } = get();
    const seen = new Set(existing.map(entryKey));
    const fresh = entries.filter((entry) => !seen.has(entryKey(entry)));
    if (fresh.length === 0) {
      return;
    }
    set({ entries: [...existing, ...fresh].slice(-MAX_ENTRIES) });
  },

  clear: async () => {
    try {
      await logClear();
    } catch {
      // 忽略清空失败。
    }
    set({ entries: [] });
  },
}));
