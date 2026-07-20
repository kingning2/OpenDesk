/**
 * YouTube API key settings IPC (React → Rust SQLite).
 */

import { invoke } from "@tauri-apps/api/core";

export interface CrawlerYoutubeApiKeyResponse {
  api_key: string;
}

/** Read persisted YouTube Data API key. */
export async function crawlerYoutubeApiKeyGet(): Promise<CrawlerYoutubeApiKeyResponse> {
  return invoke<CrawlerYoutubeApiKeyResponse>("crawler_youtube_api_key_get");
}

/** Persist YouTube Data API key. */
export async function crawlerYoutubeApiKeySet(
  apiKey: string,
): Promise<CrawlerYoutubeApiKeyResponse> {
  return invoke<CrawlerYoutubeApiKeyResponse>("crawler_youtube_api_key_set", {
    request: { api_key: apiKey },
  });
}
