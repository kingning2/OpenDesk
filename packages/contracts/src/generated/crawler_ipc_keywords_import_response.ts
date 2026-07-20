export interface CrawlerIpcKeywordsImportResponse {
  ok: boolean;
  batch_id: string;
  inserted: number;
  skipped_existing: number;
  skipped_too_long: number;
  total: number;
  trace_id?: string;
  message?: string;
}
