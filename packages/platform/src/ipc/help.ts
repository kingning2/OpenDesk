import { invokeIpc } from "./invoke";
import type { HelpIpcAskRequest, HelpIpcAskResponse } from "@desk/contracts";

/**
 * Ask the system navigation assistant one stateless question; the reply is
 * streamed back via `chat:message/token` / `chat:message/tool` events
 * (session_id `"help"`), while this promise resolves once streaming finishes.
 *
 * @author coisini
 */
export async function helpAsk(input: HelpIpcAskRequest): Promise<HelpIpcAskResponse> {
  return invokeIpc<HelpIpcAskResponse>("help_ask", { request: input });
}
