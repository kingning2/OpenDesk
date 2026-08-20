/**
 * 抖音编译期静态路由（占位 — 协议待接入）。
 *
 * @author Xiaoman
 * @created 2026-08-18
 */

import type { ComponentType } from "react";

import type { NavItem } from "../nav-registry";

/** 页面懒加载工厂。 */
export type PageLoader = () => Promise<ComponentType>;

export const routeSegments = [];

export const platformCapabilities: readonly string[] = [
  "chat",
  "auto_reply",
  "order",
  "account",
] as const;

import type { LucideIcon } from "@desk/ui/icons";

/** 平台管理导航项（各 channel 构建可扩展）。 */
export interface ManageNavItem {
  key: string;
  label: string;
  icon: LucideIcon;
  description: string;
}

export const manageNav: readonly ManageNavItem[] = [];

export const manageNavGroups: {
  label: string;
  icon?: LucideIcon;
  items: readonly ManageNavItem[];
}[] = [];

export const sidebarNavItems: NavItem[] = [];

export const pageLoaders: Record<string, PageLoader> = {};

export function manageTitleFromPath(): string | null {
  return null;
}

/** 首页管理入口（非 xianyu 构建为空）。 */
export const homeManageNav: readonly [] = [];
