import { invokeIpc } from "./invoke";
import type { ChatIpcSendRequest, ChatIpcSendResponse } from "@desk/contracts";

/**
 * Send one chat message; the reply is streamed back via `chat:message/token`
 * events, while this promise resolves once streaming finishes.
 *
 * @author coisini
 */
export async function chatSend(
  input: ChatIpcSendRequest,
): Promise<ChatIpcSendResponse> {
  return invokeIpc<ChatIpcSendResponse>("chat_send", { request: input });
}
