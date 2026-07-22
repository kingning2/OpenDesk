export interface CrawlerIpcChannelListResponse {
  ok: boolean;
  channels_json: string;
  total: number;
  trace_id?: string;
}
