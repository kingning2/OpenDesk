/**
 * 小红书编译期静态路由（占位 — 协议待接入）。
 *
 * @author Xiaoman
 * @created 2026-08-18
 */

import type { ComponentType } from "react";
import { MessageSquare } from "@desk/ui/icons";
import { CHANNEL_WORKBENCH_PATH } from "@desk/platform/compile";

import type { NavItem } from "../nav-registry";

/** 页面懒加载工厂。 */
export type PageLoader = () => Promise<ComponentType>;

function toRouteSegment(fullPath: string): string {
  return fullPath.replace(/^\//, "");
}

export const routeSegments = [{ path: toRouteSegment(CHANNEL_WORKBENCH_PATH) }];

export const platformCapabilities: readonly string[] = [
  "chat",
  "auto_reply",
  "account",
] as const;

export const sidebarNavItems: NavItem[] = [
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

export const pageLoaders: Record<string, PageLoader> = {
  [CHANNEL_WORKBENCH_PATH]: loadWorkbench,
};

export function manageTitleFromPath(): string | null {
  return null;
}

/** 首页管理入口（非 xianyu 构建为空）。 */
export const homeManageNav: readonly [] = [];
