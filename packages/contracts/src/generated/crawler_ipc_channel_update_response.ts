export interface CrawlerIpcChannelUpdateResponse {
  ok: boolean;
  id: number;
  verified_email?: string;
  email_status?: string;
  trace_id?: string;
}
