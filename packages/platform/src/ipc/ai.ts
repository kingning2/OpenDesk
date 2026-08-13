/**
 * AI 配置 IPC 封装。
 *
 * @author Xiaoman
 * @created 2026-08-13
 */

import type { AiIpcConfigRequest, AiIpcConfigResponse } from "@desk/contracts";

import { call } from "./invoke";

/**
 * 读取 AI provider 配置。
 *
 * @author Xiaoman
 * @created 2026-08-13
 *
 * @returns 当前 AI 配置
 */
export function aiConfigGet(): Promise<AiIpcConfigResponse> {
  return call<AiIpcConfigResponse>("ai_config_get");
}

/**
 * 写入 AI provider 配置。
 *
 * @author Xiaoman
 * @created 2026-08-13
 *
 * @param config - 新配置
 * @returns 写入后的配置
 */
export function aiConfigSet(config: AiIpcConfigRequest): Promise<AiIpcConfigResponse> {
  return call<AiIpcConfigResponse>("ai_config_set", { config });
}

/**
 * API Key 连通性探测结果。
 *
 * @author Xiaoman
 * @created 2026-08-13
 */
export interface AiApiKeyTestResult {
  /** 是否探测成功。 */
  ok: boolean;
  /** 结果说明。 */
  message: string;
}

/**
 * 探测 OpenAI 兼容 API Key。
 *
 * @author Xiaoman
 * @created 2026-08-13
 *
 * @param baseUrl - provider base URL
 * @param apiKey - API Key
 * @returns 探测结果
 */
export function aiTestApiKey(
  baseUrl: string,
  apiKey: string,
): Promise<AiApiKeyTestResult> {
  return call<AiApiKeyTestResult>("ai_test_api_key", { baseUrl, apiKey });
}
