/**
 * LLM Provider 设置面板（嵌入 SettingsDialog）。
 *
 * @author Xiaoman
 * @created 2026-07-22
 */

import {
  Button,
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@desk/ui";
import { useI18n } from "../../i18n";
import { LLM_VENDOR_PRESETS, type LlmVendorPresetId } from "./llm-presets";
import { useLlmSettings } from "./use-llm-settings";

/**
 * `LlmSettingsPanel` 属性。
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
export interface LlmSettingsPanelProps {
  /** 外部注入的 hook 状态（便于 Dialog 统一脏检查）。 */
  llm: ReturnType<typeof useLlmSettings>;
}

/**
 * LLM 配置表单。
 *
 * @author Xiaoman
 * @created 2026-07-22
 *
 * @param props - 见 {@link LlmSettingsPanelProps}
 * @returns 面板节点
 */
export function LlmSettingsPanel({ llm }: LlmSettingsPanelProps) {
  const { t } = useI18n();
  const {
    preset,
    applyPreset,
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
  } = llm;

  return (
    <section className="flex max-w-lg flex-col gap-6">
      <p className="text-[length:var(--text-sm)] leading-relaxed text-muted-foreground">
        {t("settings.llmDescription")}
      </p>

      <div className="flex flex-col gap-2">
        <label
          htmlFor="settings-llm-preset"
          className="text-[length:var(--text-sm)] font-medium text-foreground"
        >
          {t("settings.llmPreset")}
        </label>
        <Select
          value={preset}
          disabled={loading || saving}
          onValueChange={(value) => applyPreset(value as LlmVendorPresetId)}
        >
          <SelectTrigger id="settings-llm-preset" className="w-full max-w-md">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {LLM_VENDOR_PRESETS.map((item) => (
              <SelectItem key={item.id} value={item.id}>
                {t(item.labelKey)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      <div className="flex flex-col gap-2">
        <label
          htmlFor="settings-llm-base-url"
          className="text-[length:var(--text-sm)] font-medium text-foreground"
        >
          {t("settings.llmBaseUrl")}
        </label>
        <Input
          id="settings-llm-base-url"
          autoComplete="off"
          disabled={loading || saving}
          value={baseUrl}
          onChange={(event) => setBaseUrl(event.target.value)}
          placeholder={t("settings.llmBaseUrlPlaceholder")}
          className="max-w-md"
        />
      </div>

      <div className="flex flex-col gap-2">
        <label
          htmlFor="settings-llm-model"
          className="text-[length:var(--text-sm)] font-medium text-foreground"
        >
          {t("settings.llmModelId")}
        </label>
        <Input
          id="settings-llm-model"
          autoComplete="off"
          disabled={loading || saving}
          value={modelId}
          onChange={(event) => setModelId(event.target.value)}
          placeholder={t("settings.llmModelIdPlaceholder")}
          className="max-w-md"
        />
      </div>

      <div className="flex flex-col gap-2">
        <label
          htmlFor="settings-llm-key"
          className="text-[length:var(--text-sm)] font-medium text-foreground"
        >
          {t("settings.llmApiKey")}
        </label>
        <Input
          id="settings-llm-key"
          type="password"
          autoComplete="off"
          disabled={loading || saving}
          value={apiKey}
          onChange={(event) => setApiKey(event.target.value)}
          placeholder={
            hasApiKey
              ? t("settings.llmApiKeyKeepPlaceholder")
              : t("settings.llmApiKeyPlaceholder")
          }
          className="max-w-md"
        />
        <p className="text-[length:var(--text-xs)] text-muted-foreground">
          {t("settings.llmApiKeyHint")}
        </p>
      </div>

      <div className="flex flex-wrap items-center gap-3">
        <Button
          type="button"
          disabled={loading || saving || !dirty}
          onClick={() => void save()}
        >
          {saving ? t("settings.saving") : t("settings.save")}
        </Button>
        <Button
          type="button"
          variant="outline"
          disabled={loading || saving || testing || !modelId.trim()}
          onClick={() => void test()}
        >
          {testing ? t("settings.llmTesting") : t("settings.llmTest")}
        </Button>
        {savedMessage ? (
          <span className="text-[length:var(--text-sm)] text-emerald-600 dark:text-emerald-400">
            {t("settings.saved")}
          </span>
        ) : null}
        {!loading && configured ? (
          <span className="text-[length:var(--text-sm)] text-muted-foreground">
            {t("settings.configured")}
          </span>
        ) : null}
        {!loading && hasApiKey ? (
          <span className="text-[length:var(--text-sm)] text-muted-foreground">
            {t("settings.llmKeyStored")}
          </span>
        ) : null}
      </div>

      {testMessage ? (
        <p
          className={
            testOk
              ? "text-[length:var(--text-sm)] text-emerald-600 dark:text-emerald-400"
              : "text-[length:var(--text-sm)] text-red-500"
          }
        >
          {testMessage}
        </p>
      ) : null}

      {error ? (
        <p className="text-[length:var(--text-sm)] text-red-500">{error}</p>
      ) : null}
    </section>
  );
}
