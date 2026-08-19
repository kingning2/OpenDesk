/**
 * 工作区标签状态（打开路径 + 选择 / 关闭）。
 *
 * @author coisini
 * @created 2026-07-20
 */

import { useCallback, useContext, useEffect, useMemo, useState, createContext, type ReactNode } from "react";
import { useLocation, useNavigate } from "react-router";

import { navItems } from "../route/nav-registry";
import { getPageMeta } from "../route/page-meta";
import { getTabGroup } from "@desk/platform/compile";
import type { TabBarItem } from "./layout";

/**
 * 管理工作区多标签打开路径与导航。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @returns 标签列表与操作回调
 */
export function useWorkspaceTabs() {
  const navigate = useNavigate();
  const { pathname } = useLocation();
  const [openPaths, setOpenPaths] = useState(() => [pathname]);

  const ensureTab = useCallback((path: string) => {
    if (path === "/settings") {
      return;
    }
    setOpenPaths((current) => {
      if (current.includes(path)) {
        return current;
      }
      const group = getTabGroup(path);
      const groupIndex = current.findIndex((tabPath) => getTabGroup(tabPath) === group);
      if (groupIndex !== -1) {
        const next = [...current];
        next[groupIndex] = path;
        return next;
      }
      return [...current, path];
    });
  }, []);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      ensureTab(pathname);
    }, 0);
    return () => {
      window.clearTimeout(timer);
    };
  }, [pathname, ensureTab]);

  const tabs = useMemo<TabBarItem[]>(
    () =>
      openPaths.map((path) => {
        const navItem = navItems.find((item) => item.path === path);
        const meta = getPageMeta(path);
        return {
          id: path,
          path,
          label: navItem?.label ?? meta.title,
          closable: path !== "/",
        };
      }),
    [openPaths],
  );

  const selectTab = useCallback(
    (path: string) => {
      ensureTab(path);
      if (path !== pathname) {
        navigate(path);
      }
    },
    [ensureTab, navigate, pathname],
  );

  const closeTab = useCallback(
    (path: string) => {
      setOpenPaths((current) => {
        if (current.length === 1) {
          return current;
        }

        const index = current.findIndex((tabPath) => tabPath === path);
        if (index === -1) {
          return current;
        }

        const nextPaths = current.filter((tabPath) => tabPath !== path);
        if (pathname === path) {
          const fallback = nextPaths[Math.max(0, index - 1)] ?? nextPaths[0];
          navigate(fallback);
        }

        return nextPaths;
      });
    },
    [navigate, pathname],
  );

  const addTab = useCallback(() => {
    selectTab("/");
  }, [selectTab]);

  return {
    tabs,
    activePath: pathname,
    openPaths,
    ensureTab,
    selectTab,
    closeTab,
    addTab,
  };
}

type WorkspaceNavValue = {
  selectTab: (path: string) => void;
};

const WorkspaceNavContext = createContext<WorkspaceNavValue | null>(null);

/**
 * 向首页等嵌套页面提供带过渡的工作区导航。
 *
 * @author coisini
 * @created 2026-08-19
 */
export function WorkspaceNavProvider({
  selectTab,
  children,
}: {
  selectTab: (path: string) => void;
  children: ReactNode;
}) {
  const value = useMemo(() => ({ selectTab }), [selectTab]);
  return <WorkspaceNavContext.Provider value={value}>{children}</WorkspaceNavContext.Provider>;
}

/**
 * 读取工作区导航（含路由过渡）。
 *
 * @author coisini
 * @created 2026-08-19
 */
export function useWorkspaceNav(): WorkspaceNavValue {
  const context = useContext(WorkspaceNavContext);
  if (!context) {
    throw new Error("useWorkspaceNav must be used within WorkspaceNavProvider");
  }
  return context;
}
