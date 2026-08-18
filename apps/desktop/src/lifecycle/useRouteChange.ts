/**
 * 路由切换生命周期。
 *
 * @author Xiaoman
 * @created 2026-08-18
 */

import { useEffect, useRef } from "react";
import { useLocation } from "react-router";

import { logWrite } from "@desk/platform/ipc/log";

import { getPageMeta } from "../route/page-meta";

/**
 * 路由切换钩子 — pathname 变化时 invoke Rust 写入访问日志。
 *
 * @author Xiaoman
 * @created 2026-08-18
 */
export function useRouteChange(): void {
  const { pathname } = useLocation();
  const previousPath = useRef<string | null>(null);

  useEffect(() => {
    if (previousPath.current === pathname) {
      return;
    }
    previousPath.current = pathname;

    const meta = getPageMeta(pathname);
    const parts = [`访问页面 title=${meta.title} path=${pathname}`];
    if (meta.description) {
      parts.push(`desc=${meta.description}`);
    }
    void logWrite(parts.join(" "), "INFO").catch(() => {});
  }, [pathname]);
}
