export interface CrawlerIpcKeywordsGenerateResponse {
  ok: boolean;
  batch_id: string;
  inserted: number;
  requested: number;
  keywords_json: string;
  trace_id?: string;
  message?: string;
}
