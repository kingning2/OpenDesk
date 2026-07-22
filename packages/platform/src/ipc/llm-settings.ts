/**
 * LLM Provider 设置 IPC（React → Rust）。
 *
 * 约束：get/save 响应永不包含 api_key（含脱敏）。
 *
 * @author Xiaoman
 * @created 2026-07-22
 */

import { invoke } from "@tauri-apps/api/core";

/** 未配置 LLM 时的标准错误码（后续 AI 入口复用）。 */
export const LLM_NOT_CONFIGURED = "LLM_NOT_CONFIGURED" as const;

/** LLM 连通性探测失败错误码。 */
export const LLM_TEST_FAILED = "LLM_TEST_FAILED" as const;

/**
 * LLM 设置元数据（无密钥）。
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
export interface LlmSettingsResponse {
  /** Provider：openai / anthropic / openai_compatible */
  provider: string;
  /** 可选 Base URL */
  base_url?: string | null;
  /** 默认模型 ID */
  model_id: string;
  /** 是否具备调用条件 */
  configured: boolean;
  /** keyring 是否已有密钥 */
  has_api_key: boolean;
}

/**
 * 保存请求；`api_key` 空串表示保留 keyring 中已有密钥。
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
export interface LlmSettingsSaveRequest {
  provider: string;
  base_url?: string;
  model_id: string;
  api_key: string;
}

/**
 * 连通性探测请求。
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
export interface LlmTestConnectionRequest {
  provider: string;
  base_url?: string;
  model_id: string;
  api_key: string;
}

/**
 * 连通性探测结果。
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
export interface LlmTestConnectionResponse {
  ok: boolean;
  error_code?: string | null;
  message: string;
}

/**
 * 读取 LLM 设置元数据（不含 api_key）。
 *
 * @author Xiaoman
 * @created 2026-07-22
 *
 * @returns 当前配置快照
 */
export async function llmSettingsGet(): Promise<LlmSettingsResponse> {
  return invoke<LlmSettingsResponse>("llm_settings_get");
}

/**
 * 保存 LLM 设置；密钥写入 OS keyring，响应不回传密钥。
 *
 * @author Xiaoman
 * @created 2026-07-22
 *
 * @param request - 保存请求
 * @returns 保存后的元数据
 */
export async function llmSettingsSave(
  request: LlmSettingsSaveRequest,
): Promise<LlmSettingsResponse> {
  return invoke<LlmSettingsResponse>("llm_settings_save", { request });
}

/**
 * 测试 LLM 连接。
 *
 * @author Xiaoman
 * @created 2026-07-22
 *
 * @param request - 探测请求
 * @returns 探测结果
 */
export async function llmTestConnection(
  request: LlmTestConnectionRequest,
): Promise<LlmTestConnectionResponse> {
  return invoke<LlmTestConnectionResponse>("llm_test_connection", { request });
}

/**
 * 判断设置快照是否已可调用 LLM。
 *
 * @author Xiaoman
 * @created 2026-07-22
 *
 * @param settings - get/save 返回的元数据
 * @returns 已配置为 true
 */
export function isLlmConfigured(settings: Pick<LlmSettingsResponse, "configured">): boolean {
  return settings.configured === true;
}
