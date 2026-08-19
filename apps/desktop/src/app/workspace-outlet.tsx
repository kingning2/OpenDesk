/**
 * 工作区内容出口：按打开的标签懒加载页面，已加载的保持挂载（keep-alive）。
 *
 * @author coisini
 * @created 2026-07-20
 */

import { type ComponentType, useEffect, useLayoutEffect, useRef, useState } from "react";
import { cn, motion, PageScaffold, useAnimation, useReducedMotion } from "@desk/ui";
import { pageLoaders } from "@platform-routes";

import { needsFillLayout } from "./workspace-layout";
import { HomePage } from "./pages/home-page";

type PageLoader = () => Promise<ComponentType>;

const PAGE_LOADERS: Record<string, PageLoader> = {
  "/": async () => HomePage,
  "/features/agent": async () => {
    const { AgentPage } = await import("@feature/agent/agent-page");
    return AgentPage;
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
 * 切入：从右侧滑入。切出：立刻隐藏，无退场动画。
 *
 * @author coisini
 * @created 2026-07-21
 */
function WorkspaceTab({
  path,
  active,
  reducedMotion,
}: {
  path: string;
  active: boolean;
  reducedMotion: boolean;
}) {
  const [Page, setPage] = useState<ComponentType | null>(() => pageCache.get(path) ?? null);
  const controls = useAnimation();
  const skipFirstEnter = useRef(true);

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

  useLayoutEffect(() => {
    if (!active) {
      skipFirstEnter.current = false;
      void controls.set({ x: 0 });
      return;
    }
    if (reducedMotion || skipFirstEnter.current) {
      skipFirstEnter.current = false;
      void controls.set({ x: 0 });
      return;
    }
    void controls.set({ x: "100%" });
    void controls.start({
      x: 0,
      transition: { duration: 0.5, ease: [0.23, 1, 0.32, 1] },
    });
  }, [active, controls, reducedMotion]);

  return (
    <motion.div
      className={cn(
        "absolute inset-0 flex min-h-0 flex-col overflow-hidden will-change-transform",
        !active && "hidden pointer-events-none",
      )}
      initial={false}
      animate={controls}
      aria-hidden={!active}
    >
      {!Page ? (
        <div
          className="flex min-h-0 flex-1 items-center justify-center text-muted-foreground"
          aria-busy="true"
          role="status"
        />
      ) : (
        <PageScaffold
          fill
          scroll={!needsFillLayout(path)}
          containerWidth="full"
          className="min-h-0 flex-1"
        >
          <Page />
        </PageScaffold>
      )}
    </motion.div>
  );
}

/**
 * Tab keep-alive：已打开标签保持挂载（隐藏），长任务切换标签不中断。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @param props.openPaths - 已打开路径
 * @param props.activePath - 当前激活路径
 */
export function WorkspaceOutlet({ openPaths, activePath }: WorkspaceOutletProps) {
  const reducedMotion = useReducedMotion() ?? false;
  const paths = openPaths.includes(activePath) ? openPaths : [...openPaths, activePath];

  return (
    <div className="relative min-h-0 flex-1 overflow-hidden">
      {paths.map((path) => (
        <WorkspaceTab
          key={path}
          path={path}
          active={path === activePath}
          reducedMotion={reducedMotion}
        />
      ))}
    </div>
  );
}
