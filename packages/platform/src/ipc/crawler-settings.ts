/**
 * YouTube API key settings IPC (React → Rust SQLite).
 */

import { invokeIpc } from "./invoke";

export interface CrawlerYoutubeApiKeyResponse {
  api_key: string;
}

/** Read persisted YouTube Data API key. */
export async function crawlerYoutubeApiKeyGet(): Promise<CrawlerYoutubeApiKeyResponse> {
  return invokeIpc<CrawlerYoutubeApiKeyResponse>("crawler_youtube_api_key_get");
}

/** Persist YouTube Data API key. */
export async function crawlerYoutubeApiKeySet(
  apiKey: string,
): Promise<CrawlerYoutubeApiKeyResponse> {
  return invokeIpc<CrawlerYoutubeApiKeyResponse>("crawler_youtube_api_key_set", {
    request: { api_key: apiKey },
  });
}
