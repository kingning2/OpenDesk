export interface CrawlerDtoChannelResult {
  platform: string;
  channel_id: string;
  title: string;
  country?: string;
  subscriber_count?: number;
  email?: string;
  description?: string;
  custom_url?: string;
}
