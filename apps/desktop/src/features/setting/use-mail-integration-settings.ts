/**
 * Email-read integration settings — load, save, probe, and script test.
 *
 * @author coisini
 * @created 2026-08-01
 */

import { useCallback, useEffect, useState } from "react";
import {
  mailEmailReadIntegrationGet,
  mailEmailReadIntegrationProbe,
  mailEmailReadIntegrationSave,
  type MailEmailReadIntegrationConfig,
} from "@desk/platform/ipc/mail-integration";
import { useT } from "../../i18n";
import { runEmailReadParseScript } from "./mail-integration-parse";

/** Placeholder before IPC loads DB-backed settings (no business defaults). */
function blankMailIntegrationConfig(): MailEmailReadIntegrationConfig {
  return {
    enabled: false,
    api_base: "",
    pixel_path_template: "",
    query_path_template: "",
    parse_script: "",
  };
}

/**
 * Hook for mail open-tracking external integration settings.
 *
 * @author coisini
 * @created 2026-08-01
 */
export function useMailIntegrationSettings() {
  const t = useT();
  const [config, setConfig] = useState<MailEmailReadIntegrationConfig>(
    blankMailIntegrationConfig,
  );
  const [baseline, setBaseline] = useState<MailEmailReadIntegrationConfig>(
    blankMailIntegrationConfig,
  );
  const [loaded, setLoaded] = useState(false);
  const [probeEmail, setProbeEmail] = useState("recipient@example.com");
  const [probeTrackingId, setProbeTrackingId] = useState("0123456789abcdef0123456789abcdef");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [probing, setProbing] = useState(false);
  const [savedMessage, setSavedMessage] = useState("");
  const [probeRaw, setProbeRaw] = useState("");
  const [probeParsed, setProbeParsed] = useState("");
  const [probeError, setProbeError] = useState("");
  const [error, setError] = useState("");

  const refresh = useCallback(async () => {
    setError("");
    setLoading(true);
    try {
      const response = await mailEmailReadIntegrationGet();
      setConfig(response);
      setBaseline(response);
      setLoaded(true);
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
      const response = await mailEmailReadIntegrationSave(config);
      setConfig(response);
      setBaseline(response);
      setSavedMessage("saved");
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      throw err;
    } finally {
      setSaving(false);
    }
  }, [config]);

  const discard = useCallback(() => {
    setConfig(baseline);
    setSavedMessage("");
    setError("");
    setProbeRaw("");
    setProbeParsed("");
    setProbeError("");
  }, [baseline]);

  const runProbe = useCallback(async () => {
    setProbing(true);
    setProbeError("");
    setProbeRaw("");
    setProbeParsed("");
    try {
      const response = await mailEmailReadIntegrationProbe({
        config,
        recipient_email: probeEmail,
        tracking_id: probeTrackingId,
      });
      setProbeRaw(response.response_json);
      let data: unknown;
      try {
        data = JSON.parse(response.response_json) as unknown;
      } catch (parseError) {
        const snippet = response.response_json.slice(0, 160);
        const detail =
          parseError instanceof Error ? parseError.message : String(parseError);
        throw new Error(
          t("settings.mailIntegrationProbeInvalidJson", { detail, snippet }),
        );
      }
      const parsed = runEmailReadParseScript(config.parse_script, data);
      setProbeParsed(JSON.stringify(parsed, null, 2));
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      if (message.startsWith("parse_script_syntax:")) {
        setProbeError(
          t("settings.mailIntegrationProbeScriptSyntax", {
            detail: message.slice("parse_script_syntax:".length).trim(),
          }),
        );
      } else if (message === "parse_script_empty") {
        setProbeError(t("settings.mailIntegrationProbeScriptEmpty"));
      } else if (message === "parse_script_invalid_return") {
        setProbeError(t("settings.mailIntegrationProbeScriptInvalidReturn"));
      } else {
        setProbeError(message);
      }
    } finally {
      setProbing(false);
    }
  }, [config, probeEmail, probeTrackingId, t]);

  const dirty =
    loaded &&
    (config.enabled !== baseline.enabled ||
      config.api_base !== baseline.api_base ||
      config.pixel_path_template !== baseline.pixel_path_template ||
      config.query_path_template !== baseline.query_path_template ||
      config.parse_script !== baseline.parse_script);

  return {
    config,
    setConfig,
    loaded,
    probeEmail,
    setProbeEmail,
    probeTrackingId,
    setProbeTrackingId,
    loading,
    saving,
    probing,
    savedMessage,
    probeRaw,
    probeParsed,
    probeError,
    error,
    refresh,
    save,
    discard,
    runProbe,
    dirty,
  };
}
