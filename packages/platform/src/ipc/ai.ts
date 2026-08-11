import { invoke } from "@tauri-apps/api/core";
import type { AiIpcConfigRequest, AiIpcConfigResponse } from "@desk/contracts";

export function aiConfigGet(): Promise<AiIpcConfigResponse> {
  return invoke<AiIpcConfigResponse>("ai_config_get");
}

export function aiConfigSet(config: AiIpcConfigRequest): Promise<AiIpcConfigResponse> {
  return invoke<AiIpcConfigResponse>("ai_config_set", { config });
}

export interface AiApiKeyTestResult {
  ok: boolean;
  message: string;
}

export function aiTestApiKey(
  baseUrl: string,
  apiKey: string,
): Promise<AiApiKeyTestResult> {
  return invoke<AiApiKeyTestResult>("ai_test_api_key", { baseUrl, apiKey });
}
