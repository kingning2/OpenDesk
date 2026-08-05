export interface ChatEventTool {
  event_id: string;
  occurred_at: string;
  session_id: string;
  message_id: string;
  seq: number;
  name: string;
  arguments: string;
  ok: boolean;
  result?: string;
}
