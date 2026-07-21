/**
 * 工作区标签状态（打开路径 + 选择 / 关闭）。
 *
 * @author coisini
 * @created 2026-07-20
 */

import { useCallback, useEffect, useMemo, useState } from "react";
import { useLocation, useNavigate } from "react-router";

import { useI18n } from "../i18n";
import { navItems } from "../route/nav-registry";
import { getPageMeta } from "../route/page-meta";
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
  const { t } = useI18n();
  const [openPaths, setOpenPaths] = useState(() => [pathname]);

  const ensureTab = useCallback((path: string) => {
    if (path === "/settings") {
      return;
    }
    setOpenPaths((current) => (current.includes(path) ? current : [...current, path]));
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
          label: t(navItem?.labelKey ?? meta.titleKey),
          closable: path !== "/",
        };
      }),
    [openPaths, t],
  );

  const selectTab = useCallback(
    (path: string) => {
      ensureTab(path);
      navigate(path);
    },
    [ensureTab, navigate],
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
