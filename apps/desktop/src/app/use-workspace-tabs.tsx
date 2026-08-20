/**
 * 工作区导航 — 侧栏切换路由，无多标签页。
 *
 * @author coisini
 * @created 2026-07-20
 */

import { useCallback, useContext, useMemo, createContext, type ReactNode } from "react";
import { useLocation, useNavigate } from "react-router";

type WorkspaceNavValue = {
  selectTab: (path: string) => void;
};

const WorkspaceNavContext = createContext<WorkspaceNavValue | null>(null);

/**
 * 管理工作区当前路径与导航。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @returns 当前路径与导航回调
 */
export function useWorkspaceTabs() {
  const navigate = useNavigate();
  const { pathname } = useLocation();

  const selectTab = useCallback(
    (path: string) => {
      if (path !== pathname) {
        navigate(path);
      }
    },
    [navigate, pathname],
  );

  return {
    activePath: pathname,
    selectTab,
  };
}

/**
 * 向嵌套页面提供工作区导航。
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
 * 读取工作区导航。
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
