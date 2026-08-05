export interface ChatEventToken {
  event_id: string;
  occurred_at: string;
  session_id: string;
  message_id: string;
  seq: number;
  delta: string;
  reasoning?: string;
  done?: boolean;
  error_message?: string;
}
