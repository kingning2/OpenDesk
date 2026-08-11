/**
 * 内置 AI 平台注册表 — 平台由应用内置，用户只配置账号，无需手动添加。
 *
 * @author coisini
 * @created 2026-08-11
 */

import type { AiProvider } from "@desk/contracts";

/** 内置平台（在 {@link AiProvider} 基础上补充前端展示标记）。 */
export interface BuiltInProvider extends AiProvider {
  /** 无需账号即可使用（如本地 Ollama），为 true 时不显示账号管理。 */
  authless?: boolean;
}

/** 内置平台列表。id 稳定，用于账号归属与过滤。 */
export const BUILT_IN_PROVIDERS: BuiltInProvider[] = [
  {
    id: "deepseek",
    kind: "deepseek",
    name: "DeepSeek",
    base_url: "https://api.deepseek.com",
  },
  {
    id: "ollama",
    kind: "openai-compatible",
    name: "Ollama",
    base_url: "http://localhost:11434/v1",
    authless: true,
  },
];

/** 内置平台 id 集合。 */
export const BUILT_IN_PROVIDER_IDS = new Set(
  BUILT_IN_PROVIDERS.map((provider) => provider.id),
);
