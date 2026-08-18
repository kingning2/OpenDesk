/**
 * 侧栏导航项注册表 — 编译期平台项来自 `@platform-routes`。
 *
 * @author coisini
 * @created 2026-07-20
 */

import type { LucideIcon } from "@desk/ui/icons";
import { Home } from "@desk/ui/icons";
import { sidebarNavItems } from "@platform-routes";

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
  /**
   * 所需平台能力（小写 snake_case，如 `coupon` / `auto_reply`）。
   * 为空/缺省时全平台可见。
   */
  requiredCapabilities?: string[];
}

/**
 * 已注册侧栏导航项（首页 + 当前编译平台静态路由）。
 *
 * @author coisini
 * @created 2026-07-20
 */
export const navItems: NavItem[] = [
  { id: "home", path: "/", label: "首页", end: true, icon: Home },
  ...sidebarNavItems,
];

/**
 * 按平台能力过滤导航项。
 *
 * @author agent
 * @created 2026-08-13
 *
 * @param items - 导航项列表
 * @param capabilities - 当前平台能力集合
 * @returns 过滤后的导航项
 */
export function filterNavItemsByCapabilities(
  items: NavItem[],
  capabilities: ReadonlySet<string>,
): NavItem[] {
  return items.filter((item) => {
    const required = item.requiredCapabilities ?? [];
    return required.every((capability) => capabilities.has(capability));
  });
}
