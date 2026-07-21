/**
 * 设置弹窗 Context 定义与 hook（与 Provider 分文件，避免 fast-refresh 警告）。
 *
 * @author coisini
 * @created 2026-07-21
 */

import { createContext, useContext } from "react";

/**
 * 设置弹窗上下文值。
 *
 * @author coisini
 * @created 2026-07-21
 */
export interface SettingsDialogContextValue {
  /** 是否打开。 */
  open: boolean;
  /** 打开设置弹窗。 */
  openSettings: () => void;
  /** 关闭设置弹窗。 */
  closeSettings: () => void;
  /** 设置打开状态。 */
  setOpen: (open: boolean) => void;
}

export const SettingsDialogContext = createContext<SettingsDialogContextValue | null>(null);

/**
 * 读取设置弹窗开关。
 *
 * @author coisini
 * @created 2026-07-21
 *
 * @returns 上下文值
 * @throws 未包裹 SettingsDialogProvider 时抛错
 */
export function useSettingsDialog(): SettingsDialogContextValue {
  const ctx = useContext(SettingsDialogContext);
  if (!ctx) {
    throw new Error("useSettingsDialog must be used within SettingsDialogProvider");
  }
  return ctx;
}
