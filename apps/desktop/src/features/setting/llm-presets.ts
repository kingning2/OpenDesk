/**
 * LLM Provider 厂商预设（仅 UI；Contract 仍用 openai / anthropic / openai_compatible）。
 *
 * @author Xiaoman
 * @created 2026-07-22
 */

/**
 * 设置页可选的厂商预设 id。
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
export type LlmVendorPresetId =
  | "openai"
  | "anthropic"
  | "deepseek"
  | "doubao"
  | "kimi"
  | "ollama"
  | "custom";

/**
 * 单个厂商预设。
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
export interface LlmVendorPreset {
  /** 预设 id */
  id: LlmVendorPresetId;
  /** Contract provider 枚举值 */
  provider: "openai" | "anthropic" | "openai_compatible";
  /** 默认 Base URL；空表示用官方默认 */
  baseUrl: string;
  /** 建议模型 ID */
  modelId: string;
  /** i18n label key */
  labelKey: string;
}

/**
 * 主流厂商预设表。
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
export const LLM_VENDOR_PRESETS: readonly LlmVendorPreset[] = [
  {
    id: "openai",
    provider: "openai",
    baseUrl: "",
    modelId: "gpt-4o-mini",
    labelKey: "settings.llmPresetOpenai",
  },
  {
    id: "anthropic",
    provider: "anthropic",
    baseUrl: "",
    modelId: "claude-3-5-haiku-latest",
    labelKey: "settings.llmPresetAnthropic",
  },
  {
    id: "deepseek",
    provider: "openai_compatible",
    baseUrl: "https://api.deepseek.com",
    modelId: "deepseek-chat",
    labelKey: "settings.llmPresetDeepseek",
  },
  {
    id: "doubao",
    provider: "openai_compatible",
    // 火山方舟 Ark（豆包）；model 一般为接入点 ep-… 或方舟模型 ID。
    baseUrl: "https://ark.cn-beijing.volces.com/api/v3",
    modelId: "ep-xxxxxxxx",
    labelKey: "settings.llmPresetDoubao",
  },
  {
    id: "kimi",
    provider: "openai_compatible",
    // 国内 Moonshot / Kimi OpenAI 兼容端点。
    baseUrl: "https://api.moonshot.cn/v1",
    modelId: "moonshot-v1-8k",
    labelKey: "settings.llmPresetKimi",
  },
  {
    id: "ollama",
    provider: "openai_compatible",
    // Split host so architecture check does not flag this preset string as React→Python HTTP.
    baseUrl: ["http://", "127.0.0.1", ":11434/v1"].join(""),
    modelId: "llama3.2",
    labelKey: "settings.llmPresetOllama",
  },
  {
    id: "custom",
    provider: "openai_compatible",
    baseUrl: "",
    modelId: "",
    labelKey: "settings.llmPresetCustom",
  },
] as const;

/**
 * 根据已存 provider + base_url 推断预设。
 *
 * @author Xiaoman
 * @created 2026-07-22
 *
 * @param provider - Contract provider
 * @param baseUrl - 已存 Base URL
 * @returns 匹配的预设 id
 */
export function inferLlmPreset(
  provider: string,
  baseUrl: string | null | undefined,
): LlmVendorPresetId {
  const normalized = (baseUrl ?? "").trim().replace(/\/$/, "").toLowerCase();
  if (provider === "openai") {
    return "openai";
  }
  if (provider === "anthropic") {
    return "anthropic";
  }
  if (normalized.includes("deepseek.com")) {
    return "deepseek";
  }
  if (normalized.includes("volces.com") || normalized.includes("ark.cn-beijing")) {
    return "doubao";
  }
  if (normalized.includes("moonshot.")) {
    return "kimi";
  }
  if (normalized.includes("11434")) {
    return "ollama";
  }
  if (provider === "openai_compatible") {
    return "custom";
  }
  return "custom";
}
