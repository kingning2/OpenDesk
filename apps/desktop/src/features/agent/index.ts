import { Bot } from "@desk/ui/icons";

export const agentFeature = {
  id: "agent",
  path: "/features/agent",
  navItem: {
    id: "agent",
    path: "/features/agent",
    labelKey: "nav.agent",
    icon: Bot,
  },
};

export { useAgentPing } from "./use-agent-ping";
