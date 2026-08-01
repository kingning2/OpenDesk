/**
 * Workflow Runtime Tauri 事件订阅。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */

import type { WorkflowRuntimeEventPhase } from "@desk/contracts";
import type { UnlistenFn } from "@tauri-apps/api/event";

import { listenEvent } from "../events";

/**
 * Runtime → UI 事件 topic（与 Rust `WORKFLOW_RUNTIME_PHASE_TOPIC` 对齐）。
 *
 * @author Xiaoman
 * @created 2026-07-23
 */
export enum WorkflowRuntimeUiEvent {
  /** 相位 / 节点状态推送。 */
  Phase = "workflow_runtime:phase",
}

/**
 * 订阅 Workflow Runtime 相位事件。
 *
 * @author Xiaoman
 * @created 2026-07-23
 *
 * @param onPhase - 回调
 * @returns unlisten
 */
export async function listenWorkflowRuntimePhase(
  onPhase: (payload: WorkflowRuntimeEventPhase) => void,
): Promise<UnlistenFn> {
  return listenEvent(WorkflowRuntimeUiEvent.Phase, onPhase);
}
