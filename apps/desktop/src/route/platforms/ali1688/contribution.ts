/**
 * 1688 路由链步骤 — 仅单站 1688 构建时由 Vite 插件 import（双站走闲鱼 Hub）。
 */

import { CHANNEL_MANAGE_ROOT, managePath } from "@desk/platform/compile";

import {
  MANAGE_NAV,
  MANAGE_NAV_GROUPS,
  MANAGE_VIEW_TITLES,
  isManageView,
  manageNavItemsForGroup,
} from "@feature/platform/ali1688/manage-nav";

import type { PageLoader, PlatformRouteContribution } from "../types";

function toRouteSegment(fullPath: string): string {
  return fullPath.replace(/^\//, "");
}

const loadManageConsole: PageLoader = async () => {
  const { Ali1688ManageConsole } = await import("@feature/platform/ali1688/manage-console");
  return Ali1688ManageConsole;
};

/** 1688 路由 contribution。 */
export const ali1688RouteContribution: PlatformRouteContribution = {
  routeSegments: [
    { path: toRouteSegment(CHANNEL_MANAGE_ROOT) },
    ...MANAGE_NAV.map((item) => ({ path: toRouteSegment(managePath(item.key)) })),
  ],
  pageLoaders: {
    [CHANNEL_MANAGE_ROOT]: loadManageConsole,
    ...Object.fromEntries(
      MANAGE_NAV.map((item) => [managePath(item.key), loadManageConsole]),
    ),
  },
  manageNavGroups: MANAGE_NAV_GROUPS.map((group) => ({
    label: group.label,
    icon: group.icon,
    keys: group.keys,
    items: manageNavItemsForGroup(group),
  })).filter((group) => group.items.length > 0),
  platformCapabilities: ["account", "manage"],
  manageTitleFromPath(pathname: string): string | null {
    if (pathname === CHANNEL_MANAGE_ROOT) {
      return "首页";
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
  },
};
