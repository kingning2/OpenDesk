/**
 * AI Feature 导出。
 *
 * @author coisini
 * @created 2026-08-11
 */

import { Bot } from "@desk/ui/icons";

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

export { AiPage } from "./ai-page";
export { useAiConfigStore } from "./use-ai-config";
export type { AiConfigState, AiAccountInput } from "./use-ai-config";
