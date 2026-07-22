/**
 * 侧栏导航项注册表。
 *
 * @author coisini
 * @created 2026-07-20
 */

import type { LucideIcon } from "@desk/ui/icons";
import { Home } from "@desk/ui/icons";

import { agentFeature } from "@feature/agent";
import { chatFeature } from "@feature/chat";
import { crawlerFeature } from "@feature/crawler";
import { customerFeature } from "@feature/customer";
import { knowledgeFeature } from "@feature/knowledge";
import { mailFeature } from "@feature/mail";
import { workflowFeature } from "@feature/workflow";

/**
 * 导航项（文案用 `labelKey` 走 i18n）。
 *
 * @author coisini
 * @created 2026-07-20
 */
export interface NavItem {
  id: string;
  path: string;
  /** i18n 点分 key，如 `nav.home`。 */
  labelKey: string;
  end?: boolean;
  icon?: LucideIcon;
}

/**
 * 已注册侧栏导航项。
 *
 * @author coisini
 * @created 2026-07-20
 */
export const navItems: NavItem[] = [
  { id: "home", path: "/", labelKey: "nav.home", end: true, icon: Home },
  agentFeature.navItem,
  crawlerFeature.navItem,
  customerFeature.navItem,
  chatFeature.navItem,
  mailFeature.navItem,
  workflowFeature.navItem,
  knowledgeFeature.navItem,
];
