export interface ChatIpcSendResponse {
  ok: boolean;
  session_id: string;
  message_id: string;
  error_message?: string;
}
