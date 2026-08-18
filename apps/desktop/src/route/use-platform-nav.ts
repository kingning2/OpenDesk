/**
 * 平台能力驱动导航 hook — 编译期固定平台与能力，无运行时 IPC。
 *
 * @author agent
 * @created 2026-08-13
 */

import { useMemo } from "react";

import { getActiveChannelPlatform } from "@desk/platform/compile";
import { platformCapabilities } from "@platform-routes";

import { navItems, type NavItem, filterNavItemsByCapabilities } from "./nav-registry";

/** 平台能力加载状态。 */
export interface UsePlatformNavResult {
  /** 当前平台能力集合。 */
  capabilities: Set<string>;
  /** 过滤后的导航项。 */
  visibleNavItems: NavItem[];
  /** 当前平台标识。 */
  kind: string;
  /** 是否加载中（编译期模式恒为 false）。 */
  loading: boolean;
}

/**
 * 返回当前编译平台的导航项。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @returns 见 {@link UsePlatformNavResult}
 */
export function usePlatformNav(): UsePlatformNavResult {
  const kind = getActiveChannelPlatform();
  const capabilities = useMemo(
    () => new Set(platformCapabilities),
    [],
  );

  const visibleNavItems = useMemo(
    () => filterNavItemsByCapabilities(navItems, capabilities),
    [capabilities],
  );

  return {
    capabilities,
    visibleNavItems,
    kind,
    loading: false,
  };
}

export { navItems };
