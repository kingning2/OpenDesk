/**
 * Persisted YouTube API key — loaded from Rust SQLite via Tauri IPC.
 *
 * @author coisini
 * @created 2026-07-20
 */

import { useCallback, useEffect, useState } from "react";
import {
  crawlerYoutubeApiKeyGet,
  crawlerYoutubeApiKeySet,
} from "@desk/platform/ipc/crawler-settings";

/**
 * YouTube API 密钥设置状态与持久化。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @returns 密钥草稿、脏标记与读写操作
 */
export function useYoutubeApiKeySettings() {
  const [apiKey, setApiKey] = useState("");
  const [baseline, setBaseline] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [savedMessage, setSavedMessage] = useState("");
  const [error, setError] = useState("");

  const refresh = useCallback(async () => {
    setError("");
    setLoading(true);
    try {
      const response = await crawlerYoutubeApiKeyGet();
      const next = response.api_key ?? "";
      setApiKey(next);
      setBaseline(next);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void refresh();
    }, 0);
    return () => {
      window.clearTimeout(timer);
    };
  }, [refresh]);

  const save = useCallback(async () => {
    setError("");
    setSavedMessage("");
    setSaving(true);
    try {
      const response = await crawlerYoutubeApiKeySet(apiKey);
      const next = response.api_key ?? "";
      setApiKey(next);
      setBaseline(next);
      setSavedMessage("saved");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      throw err;
    } finally {
      setSaving(false);
    }
  }, [apiKey]);

  /**
   * 放弃未保存编辑，恢复为已持久化基线。
   *
   * @author coisini
   * @created 2026-07-21
   */
  const discard = useCallback(() => {
    setApiKey(baseline);
    setSavedMessage("");
    setError("");
  }, [baseline]);

  return {
    apiKey,
    setApiKey,
    loading,
    saving,
    savedMessage,
    error,
    refresh,
    save,
    discard,
    dirty: apiKey !== baseline,
    configured: Boolean(baseline.trim()),
  };
}
