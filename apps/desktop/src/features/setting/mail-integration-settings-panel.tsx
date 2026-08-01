/**
 * Mail open-tracking external integration settings panel.
 *
 * @author Xiaoman
 * @created 2026-08-01
 */

import { Button, Input, LoadingState, Switch } from "@desk/ui";
import { useI18n } from "../../i18n";
import { useMailIntegrationSettings } from "./use-mail-integration-settings";

/**
 * `MailIntegrationSettingsPanel` 属性。
 *
 * @author Xiaoman
 * @created 2026-08-01
 */
export interface MailIntegrationSettingsPanelProps {
  /** 外部注入 hook（便于 Dialog 统一脏检查）。 */
  integration: ReturnType<typeof useMailIntegrationSettings>;
}

/**
 * 邮件开信追踪外部服务配置 + API 探测与解析脚本测试。
 *
 * @author Xiaoman
 * @created 2026-08-01
 *
 * @param props - 见 {@link MailIntegrationSettingsPanelProps}
 * @returns 面板节点
 */
export function MailIntegrationSettingsPanel({ integration }: MailIntegrationSettingsPanelProps) {
  const { t } = useI18n();
  const {
    config,
    setConfig,
    probeEmail,
    setProbeEmail,
    probeTrackingId,
    setProbeTrackingId,
    loading,
    loaded,
    saving,
    probing,
    savedMessage,
    probeRaw,
    probeParsed,
    probeError,
    error,
    dirty,
    save,
    runProbe,
  } = integration;

  if (loading && !loaded) {
    return (
      <div className="flex min-h-48 items-center justify-center">
        <LoadingState />
      </div>
    );
  }

  return (
    <section className="flex max-w-2xl flex-col gap-6">
      <p className="text-[length:var(--text-sm)] leading-relaxed text-muted-foreground">
        {t("settings.mailIntegrationDescription")}
      </p>

      <div className="flex items-center justify-between gap-4 rounded-[var(--radius-md)] border border-border/60 bg-muted/20 px-4 py-3">
        <div className="min-w-0">
          <p className="text-[length:var(--text-sm)] font-medium text-foreground">
            {t("settings.mailIntegrationEnabled")}
          </p>
          <p className="text-[length:var(--text-xs)] text-muted-foreground">
            {t("settings.mailIntegrationEnabledHint")}
          </p>
        </div>
        <Switch
          checked={config.enabled}
          disabled={loading || saving}
          aria-label={t("settings.mailIntegrationEnabled")}
          onCheckedChange={(checked) =>
            setConfig((current) => ({ ...current, enabled: checked }))
          }
        />
      </div>

      <div className="flex flex-col gap-2">
        <label
          htmlFor="settings-mail-integration-base"
          className="text-[length:var(--text-sm)] font-medium text-foreground"
        >
          {t("settings.mailIntegrationApiBase")}
        </label>
        <Input
          id="settings-mail-integration-base"
          disabled={loading || saving}
          value={config.api_base}
          onChange={(event) =>
            setConfig((current) => ({ ...current, api_base: event.target.value }))
          }
          placeholder="https://kol-service.example.com"
          className="max-w-xl"
        />
      </div>

      <div className="flex flex-col gap-2">
        <label
          htmlFor="settings-mail-integration-pixel"
          className="text-[length:var(--text-sm)] font-medium text-foreground"
        >
          {t("settings.mailIntegrationPixelPath")}
        </label>
        <Input
          id="settings-mail-integration-pixel"
          disabled={loading || saving}
          value={config.pixel_path_template}
          onChange={(event) =>
            setConfig((current) => ({ ...current, pixel_path_template: event.target.value }))
          }
          placeholder="/api/v1/email-read/pixel?email={{email}}&mailId={{mailId}}"
          className="max-w-xl font-mono text-[length:var(--text-xs)]"
        />
        <p className="text-[length:var(--text-xs)] text-muted-foreground">
          {t("settings.mailIntegrationPathHint")}
        </p>
      </div>

      <div className="flex flex-col gap-2">
        <label
          htmlFor="settings-mail-integration-query"
          className="text-[length:var(--text-sm)] font-medium text-foreground"
        >
          {t("settings.mailIntegrationQueryPath")}
        </label>
        <Input
          id="settings-mail-integration-query"
          disabled={loading || saving}
          value={config.query_path_template}
          onChange={(event) =>
            setConfig((current) => ({ ...current, query_path_template: event.target.value }))
          }
          placeholder="/api/v1/email-read?email={{email}}&mailId={{mailId}}"
          className="max-w-xl font-mono text-[length:var(--text-xs)]"
        />
      </div>

      <div className="flex flex-col gap-2">
        <label
          htmlFor="settings-mail-integration-parse"
          className="text-[length:var(--text-sm)] font-medium text-foreground"
        >
          {t("settings.mailIntegrationParseScript")}
        </label>
        <textarea
          id="settings-mail-integration-parse"
          disabled={loading || saving}
          value={config.parse_script}
          onChange={(event) =>
            setConfig((current) => ({ ...current, parse_script: event.target.value }))
          }
          spellCheck={false}
          className="min-h-44 w-full max-w-xl rounded-[var(--radius-md)] border border-input bg-background px-3 py-2 font-mono text-[length:var(--text-xs)] leading-relaxed"
        />
        <p className="text-[length:var(--text-xs)] text-muted-foreground">
          {t("settings.mailIntegrationParseHint")}
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
        {savedMessage ? (
          <span className="text-[length:var(--text-sm)] text-emerald-600 dark:text-emerald-400">
            {t("settings.saved")}
          </span>
        ) : null}
      </div>

      {error ? <p className="text-[length:var(--text-sm)] text-red-500">{error}</p> : null}

      <div className="flex flex-col gap-4 border-t border-border/70 pt-6">
        <div>
          <p className="text-[length:var(--text-sm)] font-medium text-foreground">
            {t("settings.mailIntegrationProbeTitle")}
          </p>
          <p className="mt-1 text-[length:var(--text-xs)] text-muted-foreground">
            {t("settings.mailIntegrationProbeHint")}
          </p>
        </div>

        <div className="grid gap-4 sm:grid-cols-2">
          <div className="flex flex-col gap-2">
            <label
              htmlFor="settings-mail-probe-email"
              className="text-[length:var(--text-sm)] font-medium text-foreground"
            >
              {t("settings.mailIntegrationProbeEmail")}
            </label>
            <Input
              id="settings-mail-probe-email"
              disabled={probing}
              value={probeEmail}
              onChange={(event) => setProbeEmail(event.target.value)}
            />
          </div>
          <div className="flex flex-col gap-2">
            <label
              htmlFor="settings-mail-probe-id"
              className="text-[length:var(--text-sm)] font-medium text-foreground"
            >
              {t("settings.mailIntegrationProbeTrackingId")}
            </label>
            <Input
              id="settings-mail-probe-id"
              disabled={probing}
              value={probeTrackingId}
              onChange={(event) => setProbeTrackingId(event.target.value)}
              className="font-mono text-[length:var(--text-xs)]"
            />
          </div>
        </div>

        <Button
          type="button"
          variant="outline"
          disabled={loading || probing}
          onClick={() => void runProbe()}
        >
          {probing ? t("settings.mailIntegrationProbing") : t("settings.mailIntegrationProbeRun")}
        </Button>

        {probeError ? (
          <p className="text-[length:var(--text-sm)] text-red-500">{probeError}</p>
        ) : null}

        {probeRaw ? (
          <div className="flex flex-col gap-2">
            <p className="text-[length:var(--text-xs)] font-medium text-foreground">
              {t("settings.mailIntegrationProbeRaw")}
            </p>
            <pre
              className="max-h-40 overflow-auto rounded-[var(--radius-md)] border border-border/60 bg-muted/30 p-3 font-mono text-[length:var(--text-xs)]"
            >
              {probeRaw}
            </pre>
          </div>
        ) : null}

        {probeParsed ? (
          <div className="flex flex-col gap-2">
            <p className="text-[length:var(--text-xs)] font-medium text-foreground">
              {t("settings.mailIntegrationProbeParsed")}
            </p>
            <pre
              className="max-h-24 overflow-auto rounded-[var(--radius-md)] border border-border/60 bg-muted/30 p-3 font-mono text-[length:var(--text-xs)]"
            >
              {probeParsed}
            </pre>
          </div>
        ) : null}
      </div>
    </section>
  );
}
