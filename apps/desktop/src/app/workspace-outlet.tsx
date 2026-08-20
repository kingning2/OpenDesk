/**
 * 工作区内容出口 — 按当前路由懒加载单页。
 *
 * @author coisini
 * @created 2026-07-20
 */

import { type ComponentType, useEffect, useState } from "react";
import { CHANNEL_MANAGE_ROOT, managePath } from "@desk/platform/compile";
import { pageLoaders } from "@platform-routes";

import { HomePage } from "./pages/home-page";

type PageLoader = () => Promise<ComponentType>;

const PAGE_LOADERS: Record<string, PageLoader> = {
  "/": async () => HomePage,
  "/features/agent": async () => {
    const { AgentPage } = await import("@feature/agent/agent-page");
    return AgentPage;
  },
  "/features/ai": async () => {
    const { AiPage } = await import("@feature/ai/ai-page");
    return AiPage;
  },
  "/features/chat": async () => {
    const { ChatPage } = await import("@feature/chat/chat-page");
    return ChatPage;
  },
  "/features/knowledge": async () => {
    const { KnowledgePage } = await import("@feature/knowledge/knowledge-page");
    return KnowledgePage;
  },
  ...pageLoaders,
};

const pageCache = new Map<string, ComponentType>();

/**
 * 加载并缓存工作区页面组件。
 *
 * @author coisini
 * @created 2026-07-21
 *
 * @param path - 工作区路径
 * @returns 页面组件；未知路径返回 null
 */
async function loadWorkspacePage(path: string): Promise<ComponentType | null> {
  const cached = pageCache.get(path);
  if (cached) {
    return cached;
  }
  let loader = PAGE_LOADERS[path];
  if (!loader) {
    if (path.startsWith(`${managePath("items")}/`)) {
      loader = PAGE_LOADERS[managePath("items")];
    } else if (path.startsWith(`${CHANNEL_MANAGE_ROOT}/`)) {
      loader = PAGE_LOADERS[CHANNEL_MANAGE_ROOT];
    }
  }
  if (!loader) {
    return null;
  }
  const Page = await loader();
  pageCache.set(path, Page);
  return Page;
}

export interface WorkspaceOutletProps {
  activePath: string;
}

/**
 * 按当前路由渲染工作区页面。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @param props.activePath - 当前激活路径
 */
export function WorkspaceOutlet({ activePath }: WorkspaceOutletProps) {
  const [Page, setPage] = useState<ComponentType | null>(() => pageCache.get(activePath) ?? null);

  useEffect(() => {
    let cancelled = false;
    const cached = pageCache.get(activePath);
    if (cached) {
      setPage(() => cached);
      return;
    }

    setPage(null);
    void loadWorkspacePage(activePath).then((loaded) => {
      if (!cancelled && loaded) {
        setPage(() => loaded);
      }
    });

    return () => {
      cancelled = true;
    };
  }, [activePath]);

  if (!Page) {
    return (
      <div
        className="flex min-h-0 flex-1 items-center justify-center text-muted-foreground"
        aria-busy="true"
        role="status"
      />
    );
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      <Page />
    </div>
  );
}
