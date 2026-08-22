/**
 * 内置 AI 平台注册表 — 平台由应用内置，用户只配置账号，无需手动添加。
 *
 * @author coisini
 * @created 2026-08-11
 */

import type { AiProvider } from "@desk/contracts";
import deepseekLogo from "../../assets/deepseek.svg";
import doubaoLogo from "../../assets/doubao.svg";
import ollamaLogo from "../../assets/ollama.svg";

/** 内置平台（在 {@link AiProvider} 基础上补充前端展示标记）。 */
export interface BuiltInProvider extends AiProvider {
  /** 无需账号即可使用（如本地 Ollama），为 true 时不显示账号管理。 */
  authless?: boolean;
  /** 平台 Logo。 */
  logo: string;
  /** 默认模型输入占位。 */
  modelPlaceholder?: string;
  /** 平台说明。 */
  hint?: string;
  /** 是否支持查询余额。 */
  supportsBalance?: boolean;
}

/** 内置平台列表。id 稳定，用于账号归属与过滤。 */
export const BUILT_IN_PROVIDERS: BuiltInProvider[] = [
  {
    id: "deepseek",
    kind: "deepseek",
    name: "DeepSeek",
    base_url: "https://api.deepseek.com",
    logo: deepseekLogo,
    modelPlaceholder: "如：deepseek-chat（可选）",
    hint: "在 DeepSeek 开放平台创建 API Key。",
    supportsBalance: true,
  },
  {
    id: "doubao",
    kind: "doubao",
    name: "豆包",
    base_url: "https://ark.cn-beijing.volces.com/api/v3",
    logo: doubaoLogo,
    modelPlaceholder: "如：ep-xxxxxxxx 或 doubao-seed-1-6",
    hint: "在火山引擎方舟控制台创建 API Key；模型填写推理接入点 ID（ep- 开头）。",
  },
  {
    id: "ollama",
    kind: "openai-compatible",
    name: "Ollama",
    base_url: "http://localhost:11434/v1",
    logo: ollamaLogo,
    authless: true,
    modelPlaceholder: "如：qwen2.5（可选）",
    hint: "无需 API Key，本地运行即可使用。",
  },
];

/** 内置平台 id 集合。 */
export const BUILT_IN_PROVIDER_IDS = new Set(
  BUILT_IN_PROVIDERS.map((provider) => provider.id),
);

/** 需要账号的云平台（DeepSeek / 豆包等）。 */
export const ACCOUNT_PROVIDERS = BUILT_IN_PROVIDERS.filter((provider) => !provider.authless);
