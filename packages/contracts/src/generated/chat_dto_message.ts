export interface ChatDtoMessage {
  id: string;
  session_id: string;
  role: string;
  content: string;
  thinking?: string;
  tools_json?: string;
  seq: number;
  created_at: number;
}
