/**
 * 设置弹窗会话脏标记 — 仅 AI 账号修改时关闭需确认。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */

import { createDeskStore } from "@desk/store";

/**
 * 设置弹窗会话状态。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */
export interface SettingsSessionState {
  /** 本次打开后是否有过 AI 账号相关修改。 */
  dirty: boolean;
  /** 标记本次会话已修改。 */
  markDirty: () => void;
  /** 打开弹窗时重置脏标记。 */
  resetDirty: () => void;
}

/**
 * 设置弹窗会话 store。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */
export const useSettingsSessionStore = createDeskStore<SettingsSessionState>((set) => ({
  dirty: false,
  markDirty: () => set({ dirty: true }),
  resetDirty: () => set({ dirty: false }),
}));

/**
 * 标记设置弹窗本次会话已有 AI 账号修改。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */
export function markSettingsDirty(): void {
  useSettingsSessionStore.getState().markDirty();
}

/**
 * 重置设置弹窗脏标记。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */
export function resetSettingsDirty(): void {
  useSettingsSessionStore.getState().resetDirty();
}

/**
 * 本次设置会话是否已有修改。
 *
 * @author Xiaoman
 * @created 2026-08-19
 *
 * @returns 有修改返回 true
 */
export function isSettingsDirty(): boolean {
  return useSettingsSessionStore.getState().dirty;
}
