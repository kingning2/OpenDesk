import type { LucideIcon } from "@desk/ui/icons";
import { Home } from "@desk/ui/icons";

import { agentFeature } from "@feature/agent";
import { chatFeature } from "@feature/chat";
import { crawlerFeature } from "@feature/crawler";
import { knowledgeFeature } from "@feature/knowledge";
import { mailFeature } from "@feature/mail";

export interface NavItem {
  id: string;
  path: string;
  label: string;
  end?: boolean;
  icon?: LucideIcon;
}

export const navItems: NavItem[] = [
  { id: "home", path: "/", label: "Home", end: true, icon: Home },
  agentFeature.navItem,
  crawlerFeature.navItem,
  chatFeature.navItem,
  mailFeature.navItem,
  knowledgeFeature.navItem,
];
