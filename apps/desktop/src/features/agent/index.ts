/**
 * Agent Feature — AI 配置 + Agent 状态。
 *
 * @author Xiaoman
 */

import { Bot } from "@desk/ui/icons";

export { AiPage } from "./ai-page";
export { AiAccountCard } from "./ai-account-card";
export type { AiAccountCardProps } from "./ai-account-card";
export { AiAccountDialog } from "./ai-account-dialog";
export { useAiConfigStore } from "./use-ai-config";
export type { AiConfigState, AiAccountInput } from "./use-ai-config";
export {
  BUILT_IN_PROVIDERS,
  BUILT_IN_PROVIDER_IDS,
  ACCOUNT_PROVIDERS,
  type BuiltInProvider,
} from "./builtin-providers";
export { useAgentPing } from "./use-agent-ping";
export { AgentPage } from "./agent-page";

/** AI 配置 feature（侧栏「AI 配置」）。 */
export const aiFeature = {
  id: "ai",
  path: "/features/ai",
  navItem: {
    id: "ai",
    path: "/features/ai",
    label: "AI 配置",
    icon: Bot,
  },
};

/** Agent 状态 feature（侧栏「Agent」）。 */
export const agentFeature = {
  id: "agent",
  path: "/features/agent",
  navItem: {
    id: "agent",
    path: "/features/agent",
    label: "Agent",
    icon: Bot,
  },
};
