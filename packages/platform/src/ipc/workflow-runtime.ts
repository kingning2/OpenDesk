/**
 * Workflow Runtime IPC（start / cancel / resume / active）。
 *
 * @author coisini
 * @created 2026-07-23
 */

import { invokeIpc } from "./invoke";
import type {
  WorkflowRuntimeIpcActiveRequest,
  WorkflowRuntimeIpcActiveResponse,
  WorkflowRuntimeIpcCancelRequest,
  WorkflowRuntimeIpcCancelResponse,
  WorkflowRuntimeIpcResumeRequest,
  WorkflowRuntimeIpcResumeResponse,
  WorkflowRuntimeIpcStartRequest,
  WorkflowRuntimeIpcStartResponse,
} from "@desk/contracts";

/**
 * 可恢复实例摘要（active.instances_json 解析项）。
 *
 * @author coisini
 * @created 2026-07-23
 */
export interface WorkflowRuntimeActiveInstance {
  instance_id: string;
  state: string;
  definition_id?: string | null;
  updated_at?: string;
  heartbeat_at?: string | null;
  error_message?: string | null;
}

/**
 * 启动工作流实例。
 *
 * @author coisini
 * @created 2026-07-23
 *
 * @param request - definition_json / context_json
 * @returns 启动响应
 */
export async function workflowRuntimeStart(
  request: WorkflowRuntimeIpcStartRequest,
): Promise<WorkflowRuntimeIpcStartResponse> {
  return invokeIpc<WorkflowRuntimeIpcStartResponse>("workflow_runtime_start", { request });
}

/**
 * 取消实例。
 *
 * @author coisini
 * @created 2026-07-23
 *
 * @param request - instance_id
 * @returns 取消响应
 */
export async function workflowRuntimeCancel(
  request: WorkflowRuntimeIpcCancelRequest,
): Promise<WorkflowRuntimeIpcCancelResponse> {
  return invokeIpc<WorkflowRuntimeIpcCancelResponse>("workflow_runtime_cancel", { request });
}

/**
 * 恢复实例。
 *
 * @author coisini
 * @created 2026-07-23
 *
 * @param request - instance_id
 * @returns 恢复响应
 */
export async function workflowRuntimeResume(
  request: WorkflowRuntimeIpcResumeRequest,
): Promise<WorkflowRuntimeIpcResumeResponse> {
  return invokeIpc<WorkflowRuntimeIpcResumeResponse>("workflow_runtime_resume", { request });
}

/**
 * 查询可恢复实例列表。
 *
 * @author coisini
 * @created 2026-07-23
 *
 * @param request - 预留空对象
 * @returns 解析后的实例列表
 */
export async function workflowRuntimeActive(
  request: WorkflowRuntimeIpcActiveRequest = {},
): Promise<WorkflowRuntimeActiveInstance[]> {
  const response = await invokeIpc<WorkflowRuntimeIpcActiveResponse>("workflow_runtime_active", {
    request,
  });
  try {
    const parsed = JSON.parse(response.instances_json ?? "[]") as WorkflowRuntimeActiveInstance[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}
