/**
 * 闲鱼管理子页出口 — 按静态 URL 渲染对应业务页。
 *
 * 导航入口由应用主侧栏（仪表盘 / 账号 / 商品 / 订单）与首页卡片提供，
 * 本组件不再重复内置二级侧栏。
 *
 * @author Xiaoman
 * @created 2026-08-13
 */

import { type ComponentType } from "react";
import { useLocation } from "react-router";
import { CHANNEL_MANAGE_ROOT, managePath } from "@desk/platform/compile";
import { XianyuAccountsPage } from "./accounts";
import { XianyuAboutPage } from "./about";
import { XianyuBatchPublishPage } from "./batch-publish";
import { XianyuBlacklistPage } from "./blacklist";
import { XianyuCardsPage } from "./cards";
import { XianyuDashboardPage } from "./dashboard";
import { XianyuDisclaimerPage } from "./disclaimer";
import { XianyuFeedbackPage } from "./feedback";
import { XianyuItemsPage } from "./items";
import { XianyuItemDetailPage } from "./item-detail";
import { XianyuKeywordsPage } from "./keywords";
import { XianyuMessageFiltersPage } from "./message-filters";
import { XianyuMessageLogsPage } from "./message-logs";
import { XianyuMessageNotificationsPage } from "./message-notifications";
import { XianyuNotificationChannelsPage } from "./notification-channels";
import { XianyuOrdersPage } from "./orders";
import { XianyuPersonalSettingsPage } from "./personal-settings";
import { XianyuProductMaterialsPage } from "./product-materials";
import { XianyuProductPublishPage } from "./product-publish";
import { XianyuPublishAddressesPage } from "./publish-addresses";
import { XianyuPublishLogsPage } from "./publish-logs";
import { XianyuRiskLogsPage } from "./risk-logs";
import { XianyuTutorialPage } from "./tutorial";
import { isManageView, type ManageView } from "./manage-nav";

/** view → 页面组件映射。 */
const VIEW_PAGES: Record<ManageView, ComponentType> = {
  dashboard: XianyuDashboardPage,
  accounts: XianyuAccountsPage,
  keywords: XianyuKeywordsPage,
  items: XianyuItemsPage,
  orders: XianyuOrdersPage,
  cards: XianyuCardsPage,
  blacklist: XianyuBlacklistPage,
  filters: XianyuMessageFiltersPage,
  channels: XianyuNotificationChannelsPage,
  notifications: XianyuMessageNotificationsPage,
  logs: XianyuMessageLogsPage,
  risk: XianyuRiskLogsPage,
  materials: XianyuProductMaterialsPage,
  publish: XianyuProductPublishPage,
  "batch-publish": XianyuBatchPublishPage,
  addresses: XianyuPublishAddressesPage,
  "publish-logs": XianyuPublishLogsPage,
  settings: XianyuPersonalSettingsPage,
  tutorial: XianyuTutorialPage,
  about: XianyuAboutPage,
  disclaimer: XianyuDisclaimerPage,
  feedback: XianyuFeedbackPage,
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
  const view = viewFromPathname(pathname);
  const Page = VIEW_PAGES[view];

  return <Page />;
}
