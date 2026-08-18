/**
 * 编译期静态路由工具 — 替代 `:platform` / `:view` 动态路由。
 *
 * 路径在构建时按 `ACTIVE_CHANNEL_PLATFORM` 展开为固定字符串，
 * 便于 Tab / 导航 / 懒加载映射使用字面量路径。
 *
 * @author Xiaoman
 * @created 2026-08-18
 */

import { ACTIVE_CHANNEL_PLATFORM } from "./channel-platform";

/** 渠道工作区根路径（静态）。 */
export const CHANNEL_ROOT = "/features/channel" as const;

/** 当前构建平台的渠道工作区路径。 */
export const CHANNEL_WORKBENCH_PATH =
  `${CHANNEL_ROOT}/${ACTIVE_CHANNEL_PLATFORM}` as const;

/** 当前构建平台的管理后台根路径。 */
export const CHANNEL_MANAGE_ROOT =
  `${CHANNEL_WORKBENCH_PATH}/manage` as const;

/**
 * 构造当前平台的管理子页路径（编译期展开，无动态段）。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @param view - 管理子页 key（如 `dashboard` / `accounts`）
 * @returns 完整静态路径
 */
export function managePath(view: string): string {
  return `${CHANNEL_MANAGE_ROOT}/${view}`;
}

/**
 * 构造当前平台的渠道工作区路径。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @returns 工作区路径
 */
export function channelWorkbenchPath(): string {
  return CHANNEL_WORKBENCH_PATH;
}

/**
 * 判断路径是否属于当前平台的管理后台分组（Tab 分组用）。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @param pathname - 待判断路径
 * @returns 属于当前平台管理后台为 `true`
 */
export function isManagePath(pathname: string): boolean {
  return (
    pathname === CHANNEL_MANAGE_ROOT ||
    pathname.startsWith(`${CHANNEL_MANAGE_ROOT}/`)
  );
}

/**
 * 计算 Tab 分组键 — 管理子页共享同一 Tab。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @param path - 工作区路径
 * @returns Tab 分组键
 */
export function getTabGroup(path: string): string {
  if (isManagePath(path)) {
    return CHANNEL_MANAGE_ROOT;
  }
  if (path === CHANNEL_WORKBENCH_PATH || path.startsWith(`${CHANNEL_WORKBENCH_PATH}/`)) {
    return CHANNEL_WORKBENCH_PATH;
  }
  return path;
}
