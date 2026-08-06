import { invokeIpc } from "./invoke";
import type {
  ChatDtoMessage,
  ChatDtoSession,
  ChatIpcMessagesLoadRequest,
  ChatIpcMessagesLoadResponse,
  ChatIpcSendRequest,
  ChatIpcSendResponse,
  ChatIpcSessionCreateRequest,
  ChatIpcSessionCreateResponse,
  ChatIpcSessionDeleteRequest,
  ChatIpcSessionDeleteResponse,
  ChatIpcSessionListResponse,
  ChatIpcSessionRenameRequest,
  ChatIpcSessionRenameResponse,
} from "@desk/contracts";

/** 一个会话的列表项（多会话持久化模式）。 */
export type ChatSession = ChatDtoSession;
/** 一条已落库的完成态消息。 */
export type ChatMessage = ChatDtoMessage;

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

/**
 * List all persisted sessions (most recently updated first).
 *
 * @author coisini
 */
export async function chatSessionList(): Promise<ChatSession[]> {
  const response = await invokeIpc<ChatIpcSessionListResponse>("chat_session_list");
  try {
    const parsed = JSON.parse(response.sessions_json ?? "[]") as ChatSession[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

/**
 * Create a new session (title optional; auto-named after the first message).
 *
 * @author coisini
 */
export async function chatSessionCreate(
  input?: ChatIpcSessionCreateRequest,
): Promise<ChatSession> {
  const response = await invokeIpc<ChatIpcSessionCreateResponse>("chat_session_create", {
    request: input ?? {},
  });
  return JSON.parse(response.session_json) as ChatSession;
}

/**
 * Rename a session.
 *
 * @author coisini
 */
export async function chatSessionRename(
  input: ChatIpcSessionRenameRequest,
): Promise<ChatSession> {
  const response = await invokeIpc<ChatIpcSessionRenameResponse>("chat_session_rename", {
    request: input,
  });
  return JSON.parse(response.session_json) as ChatSession;
}

/**
 * Delete a session (cascades its messages).
 *
 * @author coisini
 */
export async function chatSessionDelete(
  input: ChatIpcSessionDeleteRequest,
): Promise<ChatIpcSessionDeleteResponse> {
  return invokeIpc<ChatIpcSessionDeleteResponse>("chat_session_delete", { request: input });
}

/**
 * Load the persisted messages of one session.
 *
 * @author coisini
 */
export async function chatMessagesLoad(
  input: ChatIpcMessagesLoadRequest,
): Promise<ChatMessage[]> {
  const response = await invokeIpc<ChatIpcMessagesLoadResponse>("chat_messages_load", {
    request: input,
  });
  try {
    const parsed = JSON.parse(response.messages_json ?? "[]") as ChatMessage[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}
