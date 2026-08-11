/**
 * 侧栏导航项注册表。
 *
 * @author coisini
 * @created 2026-07-20
 */

import type { LucideIcon } from "@desk/ui/icons";
import { Home, MessageSquare } from "@desk/ui/icons";

/**
 * 导航项（文案为中文直接文本）。
 *
 * @author coisini
 * @created 2026-07-20
 */
export interface NavItem {
  id: string;
  path: string;
  /** 导航显示文本。 */
  label: string;
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
  { id: "home", path: "/", label: "首页", end: true, icon: Home },
  {
    id: "channel",
    path: "/features/channel",
    label: "客服",
    icon: MessageSquare,
  },
];
