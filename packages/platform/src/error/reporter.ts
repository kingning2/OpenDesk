/**
 * 错误上报通道 — 可注入式全局错误 reporter。
 *
 * `invoke.ts` 等在 platform 层无法依赖 app 层的 store / UI，因此采用注入式：
 * 应用壳层通过 `setErrorReporter` 安装真实处理函数，此处仅做分类与分发。
 * 模式同 `installConsoleBridge`（一次性幂等安装）。
 *
 * @author Xiaoman
 * @created 2026-08-18
 */

import { classifyError } from "./classify";
import type { DeskError } from "./types";

/** 错误处理函数。 */
export type ErrorReporter = (error: DeskError) => void;

/** 已安装的错误处理函数（未安装时为空）。 */
let reporter: ErrorReporter | null = null;

/** 防重入守卫：reporter 内部抛错不阻塞调用方。 */
let reporting = false;

/**
 * 安装全局错误处理函数（幂等，最后一次安装生效）。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @param fn - 错误处理函数
 */
export function setErrorReporter(fn: ErrorReporter): void {
  reporter = fn;
}

/**
 * 上报错误 — 分类后交给已安装的处理函数。
 *
 * 未安装 reporter 时静默（invoke.ts 的 logIpcCompleted 已兜底打日志）。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @param error - 未知错误
 * @param command - 产生错误的 Tauri command 名（IPC 上下文）
 */
export function reportError(error: unknown, command?: string): void {
  if (reporting || !reporter) {
    return;
  }
  const deskError = classifyError(error, command);
  reporting = true;
  try {
    reporter(deskError);
  } catch {
    // reporter 自身出错不阻塞调用方。
  } finally {
    reporting = false;
  }
}
