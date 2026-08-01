import { invokeIpc } from "./invoke";
import type { AgentIpcPingRequest, AgentIpcPingResponse } from "@desk/contracts";

export async function agentPing(
  input: AgentIpcPingRequest = {},
): Promise<AgentIpcPingResponse> {
  return invokeIpc<AgentIpcPingResponse>("agent_ping", { request: input });
}
