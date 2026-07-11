import { invoke } from "@tauri-apps/api/core";
import type { AgentIpcPingRequest, AgentIpcPingResponse } from "@desk/contracts";

export async function agentPing(
  input: AgentIpcPingRequest = {},
): Promise<AgentIpcPingResponse> {
  return invoke<AgentIpcPingResponse>("agent_ping", { request: input });
}
