/**
 * YouTube API key settings IPC (React → Rust SQLite).
 */

import { invokeIpc } from "./invoke";

export interface CrawlerYoutubeAPIKeyResponse {
  api_key: string;
}

/** Read persisted YouTube Data API key. */
export async function crawlerYoutubeAPIKeyGet(): Promise<CrawlerYoutubeAPIKeyResponse> {
  return invokeIpc<CrawlerYoutubeAPIKeyResponse>("crawler_youtube_api_key_get");
}

/** Persist YouTube Data API key. */
export async function crawlerYoutubeAPIKeySet(
  apiKey: string,
): Promise<CrawlerYoutubeAPIKeyResponse> {
  return invokeIpc<CrawlerYoutubeAPIKeyResponse>("crawler_youtube_api_key_set", {
    request: { api_key: apiKey },
  });
}
