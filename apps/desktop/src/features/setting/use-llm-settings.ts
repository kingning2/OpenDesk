/**
 * LLM Provider 设置状态与持久化（密钥仅本地草稿，永不从 Rust 回读）。
 *
 * @author Xiaoman
 * @created 2026-07-22
 */

import { useCallback, useEffect, useState } from "react";
import {
  llmSettingsGet,
  llmSettingsSave,
  llmTestConnection,
} from "@desk/platform/ipc/llm-settings";
import {
  inferLlmPreset,
  LLM_VENDOR_PRESETS,
  type LlmVendorPresetId,
} from "./llm-presets";

/**
 * LLM 设置 hook。
 *
 * @author Xiaoman
 * @created 2026-07-22
 *
 * @returns 草稿状态、脏标记与读写 / 测试操作
 */
export function useLlmSettings() {
  const [preset, setPreset] = useState<LlmVendorPresetId>("openai");
  const [provider, setProvider] = useState("openai");
  const [baseUrl, setBaseUrl] = useState("");
  const [modelId, setModelId] = useState("gpt-4o-mini");
  const [apiKey, setApiKey] = useState("");
  const [baselineProvider, setBaselineProvider] = useState("openai");
  const [baselineBaseUrl, setBaselineBaseUrl] = useState("");
  const [baselineModelId, setBaselineModelId] = useState("gpt-4o-mini");
  const [hasApiKey, setHasApiKey] = useState(false);
  const [configured, setConfigured] = useState(false);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [testing, setTesting] = useState(false);
  const [savedMessage, setSavedMessage] = useState("");
  const [testMessage, setTestMessage] = useState("");
  const [testOk, setTestOk] = useState<boolean | null>(null);
  const [error, setError] = useState("");

  const refresh = useCallback(async () => {
    setError("");
    setLoading(true);
    try {
      const response = await llmSettingsGet();
      const nextProvider = response.provider || "openai";
      const nextBase = response.base_url ?? "";
      const nextModel = response.model_id || "gpt-4o-mini";
      const nextPreset = inferLlmPreset(nextProvider, nextBase);
      setProvider(nextProvider);
      setBaseUrl(nextBase);
      setModelId(nextModel);
      setPreset(nextPreset);
      setBaselineProvider(nextProvider);
      setBaselineBaseUrl(nextBase);
      setBaselineModelId(nextModel);
      setHasApiKey(response.has_api_key);
      setConfigured(response.configured);
      setApiKey("");
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

  const metaDirty =
    provider !== baselineProvider ||
    baseUrl !== baselineBaseUrl ||
    modelId !== baselineModelId;
  const keyDirty = apiKey.trim().length > 0;
  const dirty = metaDirty || keyDirty;

  /**
   * 应用厂商预设到草稿字段。
   *
   * @author Xiaoman
   * @created 2026-07-22
   *
   * @param next - 预设 id
   */
  function applyPreset(next: LlmVendorPresetId) {
    const found = LLM_VENDOR_PRESETS.find((item) => item.id === next);
    if (!found) {
      return;
    }
    setPreset(next);
    setProvider(found.provider);
    setBaseUrl(found.baseUrl);
    if (found.modelId) {
      setModelId(found.modelId);
    }
    setSavedMessage("");
    setTestMessage("");
    setTestOk(null);
  }

  const save = useCallback(async () => {
    setError("");
    setSavedMessage("");
    setSaving(true);
    try {
      const response = await llmSettingsSave({
        provider,
        base_url: baseUrl.trim() || undefined,
        model_id: modelId.trim(),
        api_key: apiKey,
      });
      setBaselineProvider(response.provider);
      setBaselineBaseUrl(response.base_url ?? "");
      setBaselineModelId(response.model_id);
      setProvider(response.provider);
      setBaseUrl(response.base_url ?? "");
      setModelId(response.model_id);
      setHasApiKey(response.has_api_key);
      setConfigured(response.configured);
      setApiKey("");
      setSavedMessage("saved");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      throw err;
    } finally {
      setSaving(false);
    }
  }, [apiKey, baseUrl, modelId, provider]);

  const test = useCallback(async () => {
    setError("");
    setTestMessage("");
    setTestOk(null);
    setTesting(true);
    try {
      const response = await llmTestConnection({
        provider,
        base_url: baseUrl.trim() || undefined,
        model_id: modelId.trim(),
        api_key: apiKey,
      });
      setTestOk(response.ok);
      setTestMessage(response.message);
      if (!response.ok && response.error_code) {
        setError(response.error_code);
      }
    } catch (err) {
      setTestOk(false);
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setTesting(false);
    }
  }, [apiKey, baseUrl, modelId, provider]);

  const discard = useCallback(() => {
    setProvider(baselineProvider);
    setBaseUrl(baselineBaseUrl);
    setModelId(baselineModelId);
    setPreset(inferLlmPreset(baselineProvider, baselineBaseUrl));
    setApiKey("");
    setSavedMessage("");
    setTestMessage("");
    setTestOk(null);
    setError("");
  }, [baselineBaseUrl, baselineModelId, baselineProvider]);

  return {
    preset,
    applyPreset,
    provider,
    setProvider,
    baseUrl,
    setBaseUrl,
    modelId,
    setModelId,
    apiKey,
    setApiKey,
    hasApiKey,
    configured,
    loading,
    saving,
    testing,
    savedMessage,
    testMessage,
    testOk,
    error,
    dirty,
    save,
    test,
    discard,
    refresh,
  };
}
