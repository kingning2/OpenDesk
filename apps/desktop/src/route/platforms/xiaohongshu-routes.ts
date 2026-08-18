/**
 * 小红书编译期静态路由（占位 — 协议待接入）。
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
  "account",
] as const;

export const sidebarNavItems: NavItem[] = [];

export const pageLoaders: Record<string, PageLoader> = {};

export function manageTitleFromPath(): string | null {
  return null;
}

/** 首页管理入口（非 xianyu 构建为空）。 */
export const homeManageNav: readonly [] = [];
