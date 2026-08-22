/**
 * 闲鱼编译期静态路由 — 路由段、页面加载器、侧栏导航。
 *
 * 仅 `DINGDA_CHANNEL_PLATFORM=xianyu` 构建时通过 `@platform-routes` 引入。
 *
 * @author Xiaoman
 * @created 2026-08-18
 */

import type { ComponentType } from "react";
import { CHANNEL_MANAGE_ROOT, managePath } from "@desk/platform/compile";

import {
  MANAGE_NAV,
  MANAGE_NAV_GROUPS,
  manageNavItemsForGroup,
  MANAGE_VIEW_TITLES,
  isManageView,
  type ManageNavItem,
  type ManageNavGroup,
  type ManageView,
} from "@feature/xianyu/manage-nav";

import type { NavItem } from "../nav-registry";

/** 页面懒加载工厂。 */
export type PageLoader = () => Promise<ComponentType>;

/**
 * 去掉 leading slash，供 react-router child path 使用。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @param fullPath - 完整路径
 * @returns 路由段
 */
function toRouteSegment(fullPath: string): string {
  return fullPath.replace(/^\//, "");
}

/** 编译期静态路由段（无 `:platform` / `:view`）。 */
export const routeSegments = [
  { path: toRouteSegment(CHANNEL_MANAGE_ROOT) },
  ...MANAGE_NAV.map((item) => ({ path: toRouteSegment(managePath(item.key)) })),
  { path: `${toRouteSegment(managePath("items"))}/:itemId` },
];

/** 当前平台能力（编译期固定，替代运行时 IPC 过滤）。 */
export const platformCapabilities: readonly string[] = [
  "chat",
  "auto_reply",
  "coupon",
  "auto_delivery",
  "product_publish",
  "distribution",
  "order",
  "rate",
  "listing_monitor",
  "account",
  "manage",
] as const;

/** 编译期平台管理导航（侧栏数据源）。 */
export const manageNav = MANAGE_NAV;

/** 侧栏分组（含解析后的 items 与分组图标；空组不展示）。 */
export const manageNavGroups = MANAGE_NAV_GROUPS.map((group) => ({
  label: group.label,
  icon: group.icon,
  items: manageNavItemsForGroup(group),
})).filter((group) => group.items.length > 0);

/** @deprecated 使用 manageNavGroups；保留空数组兼容 nav-registry */
export const sidebarNavItems: NavItem[] = [];

const loadManageConsole: PageLoader = async () => {
  const { XianyuManageConsole } = await import("@feature/xianyu/manage-console");
  return XianyuManageConsole;
};

/** 静态路径 → 页面加载器映射。 */
export const pageLoaders: Record<string, PageLoader> = {
  [CHANNEL_MANAGE_ROOT]: loadManageConsole,
  ...Object.fromEntries(
    MANAGE_NAV.map((item) => [managePath(item.key), loadManageConsole]),
  ),
};

export { MANAGE_VIEW_TITLES, isManageView, type ManageNavItem, type ManageNavGroup, type ManageView };

/**
 * 按静态路径解析管理子页标题。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @param pathname - 当前路径
 * @returns 子页标题；非管理子页为 `null`
 */
export function manageTitleFromPath(pathname: string): string | null {
  if (pathname === CHANNEL_MANAGE_ROOT) {
    return "首页";
  }
  const prefix = `${CHANNEL_MANAGE_ROOT}/`;
  if (!pathname.startsWith(prefix)) {
    return null;
  }
  const rest = pathname.slice(prefix.length);
  if (rest.startsWith("items/") && rest.split("/").length >= 2) {
    return "商品详情";
  }
  const view = rest.split("/")[0] ?? "";
  if (isManageView(view)) {
    return MANAGE_VIEW_TITLES[view];
  }
  return null;
}
