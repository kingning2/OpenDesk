/**
 * 插件下载进度事件订阅。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */

import type { PluginEventProgress } from "@desk/contracts";

import { listenEvent } from "./index";

/** 与 Rust `PLUGIN_PROGRESS_TOPIC` 对齐（Tauri 事件名禁止 `.`）。 */
export const PLUGIN_PROGRESS_EVENT = "plugin/progress";

/**
 * 订阅插件下载进度。
 *
 * @author Xiaoman
 * @created 2026-08-19
 *
 * @param handler - 进度回调
 * @returns 取消订阅函数
 */
export function listenPluginProgress(
  handler: (payload: PluginEventProgress) => void,
): Promise<() => void> {
  return listenEvent<PluginEventProgress>(PLUGIN_PROGRESS_EVENT, handler);
}
