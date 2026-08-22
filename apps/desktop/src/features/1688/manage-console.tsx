/**
 * 1688 管理子页出口 — 按静态 URL 渲染对应业务页。
 *
 * @author Xiaoman
 * @created 2026-08-22
 */

import { type ComponentType } from "react";
import { useLocation } from "react-router";
import { CHANNEL_MANAGE_ROOT } from "@desk/platform/compile";
import { Ali1688AccountsPage } from "./accounts";
import { Ali1688SearchPage } from "./search";
import { isManageView, type ManageView } from "./manage-nav";

const VIEW_PAGES: Record<ManageView, ComponentType> = {
  accounts: Ali1688AccountsPage,
  search: Ali1688SearchPage,
};

function viewFromPathname(pathname: string): ManageView {
  if (pathname === CHANNEL_MANAGE_ROOT) {
    return "search";
  }
  const prefix = `${CHANNEL_MANAGE_ROOT}/`;
  if (!pathname.startsWith(prefix)) {
    return "search";
  }
  const segment = pathname.slice(prefix.length).split("/")[0] ?? "";
  return isManageView(segment) ? segment : "search";
}

/**
 * 1688 管理子页出口（静态 URL → 业务页）。
 */
export function Ali1688ManageConsole() {
  const { pathname } = useLocation();
  const view = viewFromPathname(pathname);
  const Page = VIEW_PAGES[view] ?? Ali1688SearchPage;
  return <Page />;
}
