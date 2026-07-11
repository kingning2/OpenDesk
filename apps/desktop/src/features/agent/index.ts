import { Bot } from "@desk/ui/icons";

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

export { AgentPage } from "./agent-page";
export { useAgentPing } from "./use-agent-ping";
