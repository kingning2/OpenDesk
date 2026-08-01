export interface CrawlerIpcKeywordsGenerateRequest {
  trace_id?: string;
  directions: string;
  languages: string;
  count_per_language: number;
  batch_id?: string;
}
