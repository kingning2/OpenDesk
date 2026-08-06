export interface KnowledgeEventDownloadProgress {
  tool: string;
  bytes_downloaded: number;
  bytes_total: number;
  status: string;
  error_message?: string;
}
