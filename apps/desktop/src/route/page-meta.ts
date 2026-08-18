/**
 * 路由 → 页面元信息（中文标题 / 描述）。
 *
 * @author coisini
 * @created 2026-07-20
 */

import {
  CHANNEL_MANAGE_ROOT,
} from "@desk/platform/compile";
import { manageTitleFromPath } from "@platform-routes";

/**
 * 页面元信息。
 *
 * @author coisini
 * @created 2026-07-20
 */
export interface PageMeta {
  /** 页面标题。 */
  title: string;
  /** 页面描述。 */
  description?: string;
}

const pageMetaByPath: Record<string, PageMeta> = {
  "/": {
    title: "首页",
    description: "DingDa 架构脚手架",
  },
  "/features/agent": {
    title: "Agent",
    description: "Sidecar 连通性垂直切片",
  },
  "/features/chat": {
    title: "Chat",
    description: "客户会话工作区",
  },
  [CHANNEL_MANAGE_ROOT]: {
    title: "管理后台",
    description: "平台业务管理控制台",
  },
  "/features/knowledge": {
    title: "Knowledge",
    description: "知识库与检索",
  },
};

/**
 * 按路径取页面元信息。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @param pathname - 路由路径
 * @returns 元信息；未知路径回退到应用名
 */
export function getPageMeta(pathname: string): PageMeta {
  const manageTitle = manageTitleFromPath(pathname);
  if (manageTitle) {
    return {
      title: manageTitle,
      description: "平台管理业务子页面",
    };
  }

  return pageMetaByPath[pathname] ?? { title: "DingDa" };
}
