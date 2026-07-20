/**
 * Persisted YouTube API key — loaded from Rust SQLite via Tauri IPC.
 */

import { useCallback, useEffect, useState } from "react";
import {
  crawlerYoutubeApiKeyGet,
  crawlerYoutubeApiKeySet,
} from "@desk/platform/ipc/crawler-settings";

export function useYoutubeApiKeySettings() {
  const [apiKey, setApiKey] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [savedMessage, setSavedMessage] = useState("");
  const [error, setError] = useState("");

  const refresh = useCallback(async () => {
    setError("");
    setLoading(true);
    try {
      const response = await crawlerYoutubeApiKeyGet();
      setApiKey(response.api_key ?? "");
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
      setApiKey(response.api_key ?? "");
      setSavedMessage("已保存");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSaving(false);
    }
  }, [apiKey]);

  return {
    apiKey,
    setApiKey,
    loading,
    saving,
    savedMessage,
    error,
    refresh,
    save,
    configured: Boolean(apiKey.trim()),
  };
}
