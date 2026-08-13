/**
 * Agent IPC 封装。
 *
 * @author Xiaoman
 * @created 2026-08-13
 */

import type { AgentIpcPingRequest, AgentIpcPingResponse } from "@desk/contracts";

import { call } from "./invoke";

/**
 * 探测 sidecar（agent ping）。
 *
 * @author Xiaoman
 * @created 2026-08-13
 *
 * @param input - ping 请求（可选 trace）
 * @returns ping 响应
 */
export async function agentPing(
  input: AgentIpcPingRequest = {},
): Promise<AgentIpcPingResponse> {
  return call<AgentIpcPingResponse>("agent_ping", { request: input });
}
