/**
 * runtime 事件订阅 — 后端运行时错误 / 生命周期事件推送。
 *
 * @author Xiaoman
 * @created 2026-08-18
 */

import type { RuntimeEventError, RuntimeEventSidecarRestarted } from "@desk/contracts";

import { listenEvent } from "./index";

/** 后端运行时错误事件主题（与 Rust `RUNTIME_ERROR_TOPIC` 对齐）。 */
export const RUNTIME_ERROR_EVENT = "runtime.error";

/** 后端侧车重启事件主题（与 Rust `SIDECAR_RESTARTED_TOPIC` 对齐）。 */
export const SIDECAR_RESTARTED_EVENT = "runtime.sidecar.restarted";

/** 订阅后端运行时错误事件；返回取消订阅函数。 */
export function listenRuntimeError(
  handler: (payload: RuntimeEventError) => void,
): Promise<() => void> {
  return listenEvent<RuntimeEventError>(RUNTIME_ERROR_EVENT, handler);
}

/** 订阅后端侧车重启事件（恢复信号）；返回取消订阅函数。 */
export function listenSidecarRestarted(
  handler: (payload: RuntimeEventSidecarRestarted) => void,
): Promise<() => void> {
  return listenEvent<RuntimeEventSidecarRestarted>(
    SIDECAR_RESTARTED_EVENT,
    handler,
  );
}
