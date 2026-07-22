/**
 * 工作区内容出口：按打开的标签懒加载页面，已加载的保持挂载（keep-alive）。
 *
 * @author coisini
 * @created 2026-07-20
 */

import { type ComponentType, useEffect, useState } from "react";
import { cn, LoadingState } from "@desk/ui";
import { useT } from "../i18n";
import { HomePage } from "./pages/home-page";

type PageLoader = () => Promise<ComponentType>;

const PAGE_LOADERS: Record<string, PageLoader> = {
  "/": async () => HomePage,
  "/features/agent": async () => {
    const { AgentPage } = await import("@feature/agent/agent-page");
    return AgentPage;
  },
  "/features/crawler": async () => {
    const { CrawlerPage } = await import("@feature/crawler/crawler-page");
    return CrawlerPage;
  },
  "/features/crawler-results": async () => {
    const { CrawlerResultsPage } = await import("@feature/crawler-results/crawler-results-page");
    return CrawlerResultsPage;
  },
  "/features/customer": async () => {
    const { CustomerPage } = await import("@feature/customer/customer-page");
    return CustomerPage;
  },
  "/features/chat": async () => {
    const { ChatPage } = await import("@feature/chat/chat-page");
    return ChatPage;
  },
  "/features/mail": async () => {
    const { MailPage } = await import("@feature/mail/mail-page");
    return MailPage;
  },
  "/features/workflow": async () => {
    const { WorkflowPage } = await import("@feature/workflow/workflow-page");
    return WorkflowPage;
  },
  "/features/knowledge": async () => {
    const { KnowledgePage } = await import("@feature/knowledge/knowledge-page");
    return KnowledgePage;
  },
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
  const loader = PAGE_LOADERS[path];
  if (!loader) {
    return null;
  }
  const Page = await loader();
  pageCache.set(path, Page);
  return Page;
}

export interface WorkspaceOutletProps {
  openPaths: string[];
  activePath: string;
}

/**
 * 单个标签页：首次打开时懒加载，之后保持挂载。
 *
 * @author coisini
 * @created 2026-07-21
 *
 * @param props.path - 页面路径
 * @param props.active - 是否为当前激活标签
 */
function WorkspaceTab({ path, active }: { path: string; active: boolean }) {
  const t = useT();
  const [Page, setPage] = useState<ComponentType | null>(() => pageCache.get(path) ?? null);

  useEffect(() => {
    let cancelled = false;
    if (Page) {
      return;
    }
    void loadWorkspacePage(path).then((loaded) => {
      if (!cancelled && loaded) {
        setPage(() => loaded);
      }
    });
    return () => {
      cancelled = true;
    };
  }, [path, Page]);

  if (!Page) {
    return (
      <div
        className={cn(
          "min-h-0 flex-1 items-center justify-center",
          active ? "flex" : "hidden",
        )}
        aria-hidden={!active}
      >
        <LoadingState label={t("loading")} size="lg" />
      </div>
    );
  }

  return (
    <div
      className={cn(
        "min-h-0 flex-1 flex-col overflow-hidden",
        active ? "flex" : "hidden",
      )}
      aria-hidden={!active}
    >
      <Page />
    </div>
  );
}

/**
 * Tab keep-alive：已打开标签保持挂载（隐藏），长任务（如爬虫轮询）切换标签不中断。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @param props.openPaths - 已打开路径
 * @param props.activePath - 当前激活路径
 */
export function WorkspaceOutlet({ openPaths, activePath }: WorkspaceOutletProps) {
  return (
    <>
      {openPaths.map((path) => (
        <WorkspaceTab key={path} path={path} active={path === activePath} />
      ))}
    </>
  );
}
