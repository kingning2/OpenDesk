/**
 * 闲鱼编译期静态路由 — 路由段、页面加载器、侧栏导航。
 *
 * 仅 `OPENDESK_CHANNEL_PLATFORM=xianyu` 构建时通过 `@platform-routes` 引入。
 *
 * @author Xiaoman
 * @created 2026-08-18
 */

import type { ComponentType } from "react";
import {
  CHANNEL_MANAGE_ROOT,
  CHANNEL_WORKBENCH_PATH,
  managePath,
} from "@desk/platform/compile";
import { LayoutDashboard, MessageSquare, Package, ShoppingCart, Users } from "@desk/ui/icons";

import {
  MANAGE_NAV,
  MANAGE_VIEW_TITLES,
  HOME_MANAGE_NAV,
  isManageView,
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
  { path: toRouteSegment(CHANNEL_WORKBENCH_PATH) },
  { path: toRouteSegment(CHANNEL_MANAGE_ROOT) },
  ...MANAGE_NAV.map((item) => ({ path: toRouteSegment(managePath(item.key)) })),
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

/** 侧栏平台相关导航项。 */
export const sidebarNavItems: NavItem[] = [
  {
    id: "platform-dashboard",
    path: managePath("dashboard"),
    label: "仪表盘",
    icon: LayoutDashboard,
    requiredCapabilities: ["manage"],
  },
  {
    id: "platform-accounts",
    path: managePath("accounts"),
    label: "账号管理",
    icon: Users,
    requiredCapabilities: ["manage"],
  },
  {
    id: "platform-items",
    path: managePath("items"),
    label: "商品管理",
    icon: Package,
    requiredCapabilities: ["manage"],
  },
  {
    id: "platform-orders",
    path: managePath("orders"),
    label: "订单管理",
    icon: ShoppingCart,
    requiredCapabilities: ["manage"],
  },
  {
    id: "channel-workbench",
    path: CHANNEL_WORKBENCH_PATH,
    label: "会话工作台",
    icon: MessageSquare,
    end: true,
    requiredCapabilities: ["chat"],
  },
];

const loadWorkbench: PageLoader = async () => {
  const { ChannelWorkbench } = await import("@feature/channel/channel-workbench");
  return ChannelWorkbench;
};

const loadManageConsole: PageLoader = async () => {
  const { XianyuManageConsole } = await import("@feature/xianyu/manage-console");
  return XianyuManageConsole;
};

/** 静态路径 → 页面加载器映射。 */
export const pageLoaders: Record<string, PageLoader> = {
  [CHANNEL_WORKBENCH_PATH]: loadWorkbench,
  [CHANNEL_MANAGE_ROOT]: loadManageConsole,
  ...Object.fromEntries(
    MANAGE_NAV.map((item) => [managePath(item.key), loadManageConsole]),
  ),
};

export { MANAGE_VIEW_TITLES, isManageView, HOME_MANAGE_NAV as homeManageNav, type ManageView };

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
    return "管理后台";
  }
  const prefix = `${CHANNEL_MANAGE_ROOT}/`;
  if (!pathname.startsWith(prefix)) {
    return null;
  }
  const view = pathname.slice(prefix.length).split("/")[0] ?? "";
  if (isManageView(view)) {
    return MANAGE_VIEW_TITLES[view];
  }
  return null;
}
