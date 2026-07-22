export interface CrawlerIpcChannelListRequest {
  trace_id?: string;
  search?: string;
  keyword?: string;
  country?: string;
  has_email?: boolean;
  email_status?: string;
  limit?: number;
  offset?: number;
}
