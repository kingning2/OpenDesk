/**
 * 1688 管理子页面导航数据 — 供 `ali1688-routes.ts` 与共享侧栏使用。
 *
 * 1688 站点仅提供账号管理（Cookie 扫码登录），无闲鱼业务子页。
 * 与 `@feature/xianyu/manage-nav` 平级、互不引用。
 *
 * @author Xiaoman
 * @created 2026-08-22
 */

import type { LucideIcon } from "@desk/ui/icons";
import { Search, ShoppingCart, Users } from "@desk/ui/icons";

/** 1688 管理子页面标识（URL 片段，如 `/manage/accounts`）。 */
export type ManageView = "accounts" | "search";

/** 管理子页面导航项。 */
export interface ManageNavItem {
  key: ManageView;
  label: string;
  icon: LucideIcon;
  description: string;
  ready: boolean;
}

/** 全部 1688 管理子页面。 */
export const MANAGE_NAV: ManageNavItem[] = [
  {
    key: "accounts",
    label: "账号管理",
    icon: Users,
    description: "1688 / 手机淘宝 Cookie 扫码登录与账号状态",
    ready: true,
  },
  {
    key: "search",
    label: "商品搜索",
    icon: Search,
    description: "Camoufox 指纹浏览器搜索 1688 批发商品",
    ready: true,
  },
];

/** 侧栏分组（Grouped Sidebar 模式）。 */
export interface ManageNavGroup {
  label: string;
  icon: LucideIcon;
  keys: ManageView[];
}

/** 侧栏分组导航。 */
export const MANAGE_NAV_GROUPS: ManageNavGroup[] = [
  { label: "交易", icon: ShoppingCart, keys: ["search", "accounts"] },
];

/**
 * 解析分组内已接入的导航项（保持 MANAGE_NAV 中的顺序）。
 *
 * @author Xiaoman
 * @created 2026-08-22
 */
export function manageNavItemsForGroup(group: ManageNavGroup): ManageNavItem[] {
  const byKey = new Map(MANAGE_NAV.map((item) => [item.key, item]));
  return group.keys
    .map((key) => byKey.get(key))
    .filter((item): item is ManageNavItem => item != null && item.ready);
}

/** 管理子页面 URL 片段 → 中文标题（标签页 / 面包屑）。 */
export const MANAGE_VIEW_TITLES: Record<ManageView, string> = Object.fromEntries(
  MANAGE_NAV.map((item) => [item.key, item.label]),
) as Record<ManageView, string>;

/** 判断字符串是否为合法管理子页面 key。 */
export function isManageView(value: string): value is ManageView {
  return value in MANAGE_VIEW_TITLES;
}

/** 构造管理子页面完整路径（编译期静态，无 platform 参数）。 */
export { managePath } from "@desk/platform/compile";
