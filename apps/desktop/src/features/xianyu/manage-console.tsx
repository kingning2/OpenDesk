/**
 * 闲鱼管理子页出口 — 按静态 URL 渲染对应业务页。
 *
 * 导航入口由应用主侧栏（首页 / 账号 / 商品 / 订单）提供；
 * 风控日志与免责声明在应用设置弹窗中。
 * 本组件不再重复内置二级侧栏。
 *
 * @author Xiaoman
 * @created 2026-08-13
 */

import { type ComponentType } from "react";
import { useLocation } from "react-router";
import { CHANNEL_MANAGE_ROOT, managePath } from "@desk/platform/compile";
import { Ali1688SearchPage } from "@feature/1688/search";
import { XianyuMonitorPage } from "./monitor";
import { XianyuMonitorRunDetailPage } from "./monitor-run-detail";
import { XianyuSearchPage } from "./search";
import { XianyuAccountsPage } from "./accounts";
import { XianyuDashboardPage } from "./dashboard";
import { XianyuItemsPage } from "./items";
import { XianyuItemDetailPage } from "./item-detail";
import { XianyuOrdersPage } from "./orders";
import { isManageView, type ManageView } from "./manage-nav";

/**
 * 兼容旧路由 `/manage/accounts-1688`：打开账号管理并定位 1688 Tab。
 * 双站构建时共享 Hub 同时含闲鱼 / 1688 Tab。
 *
 * @author Xiaoman
 * @created 2026-08-22
 */
function Accounts1688View() {
  return <XianyuAccountsPage initialTab="ali1688" />;
}

/**
 * view → 页面组件映射。
 * 1688 子页仅双站构建时注册（编译期裁剪，单站构建无该路由）。
 */
const VIEW_PAGES: Partial<Record<ManageView, ComponentType>> = {
  dashboard: XianyuDashboardPage,
  accounts: XianyuAccountsPage,
  search: XianyuSearchPage,
  monitor: XianyuMonitorPage,
  ...(__DINGDA_HAS_ALI1688__
    ? { "accounts-1688": Accounts1688View, "search-1688": Ali1688SearchPage }
    : {}),
  items: XianyuItemsPage,
  orders: XianyuOrdersPage,
};

/**
 * 从静态路径解析管理子页 key。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @param pathname - 当前路径
 * @returns 合法子页 key；缺省 / 非法时回退仪表盘
 */
function viewFromPathname(pathname: string): ManageView {
  if (pathname === CHANNEL_MANAGE_ROOT) {
    return "dashboard";
  }
  const prefix = `${CHANNEL_MANAGE_ROOT}/`;
  if (!pathname.startsWith(prefix)) {
    return "dashboard";
  }
  const segment = pathname.slice(prefix.length).split("/")[0] ?? "";
  return isManageView(segment) ? segment : "dashboard";
}

/**
 * 从路径解析商品详情 ID（`/manage/items/:itemId`）。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
function itemIdFromPathname(pathname: string): string | null {
  const prefix = `${managePath("items")}/`;
  if (!pathname.startsWith(prefix)) {
    return null;
  }
  const segment = pathname.slice(prefix.length).split("/")[0] ?? "";
  return segment ? decodeURIComponent(segment) : null;
}

/**
 * 从路径解析监控运行详情 ID（`/manage/.../monitor/runs/:runId`）。
 *
 * @author Xiaoman
 * @created 2026-08-22
 */
function monitorRunIdFromPathname(pathname: string): string | null {
  const prefix = `${managePath("monitor")}/runs/`;
  if (!pathname.startsWith(prefix)) {
    return null;
  }
  const segment = pathname.slice(prefix.length).split("/")[0] ?? "";
  return segment ? decodeURIComponent(segment) : null;
}

/**
 * 闲鱼管理子页出口（静态 URL → 业务页）。
 *
 * @author agent
 * @created 2026-08-13
 *
 * @returns 当前子页内容
 */
export function XianyuManageConsole() {
  const { pathname } = useLocation();
  const itemId = itemIdFromPathname(pathname);
  if (itemId) {
    return <XianyuItemDetailPage itemId={itemId} />;
  }
  const runId = monitorRunIdFromPathname(pathname);
  if (runId) {
    return <XianyuMonitorRunDetailPage runId={runId} />;
  }
  const view = viewFromPathname(pathname);
  const Page = VIEW_PAGES[view] ?? XianyuDashboardPage;

  return <Page />;
}
