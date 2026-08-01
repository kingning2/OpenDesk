export interface MailIpcGenerateHtmlResponse {
  ok: boolean;
  html: string;
  notes?: string;
  message: string;
  trace_id?: string;
}
