import { useCallback, useEffect, useState } from "react";
import { useLocation, useNavigate } from "react-router";
import type { TabBarItem } from "@desk/ui";

import { navItems } from "../route/nav-registry";
import { getPageMeta } from "../route/page-meta";

function createTab(path: string): TabBarItem {
  const navItem = navItems.find((item) => item.path === path);
  const meta = getPageMeta(path);

  return {
    id: path,
    path,
    label: navItem?.label ?? meta.title,
    closable: path !== "/",
  };
}

export function useWorkspaceTabs() {
  const navigate = useNavigate();
  const { pathname } = useLocation();
  const [openPaths, setOpenPaths] = useState(() => [pathname]);

  const ensureTab = useCallback((path: string) => {
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

  const tabs = openPaths.map(createTab);

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
