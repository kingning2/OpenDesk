export interface ChatIpcSendRequest {
  trace_id?: string;
  message_id?: string;
  session_id: string;
  messages_json: string;
  text: string;
}
