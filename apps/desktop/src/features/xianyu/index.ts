/**
 * 闲鱼业务 feature — 原前端管理后台页面迁入。
 *
 * 迁移策略：
 * - 公共组件 / 工具 → `@desk/ui` / `@desk/utils`（已抽取）
 * - 数据访问 → Tauri IPC（`@desk/platform/ipc/*`，复用 crates/app Rust 业务）
 * - 页面 → 按原前端结构与交互风格重写，按业务模块拆分
 */

export { XianyuAccountsPage } from "./accounts";
export { XianyuAboutPage } from "./about";
export { XianyuBatchPublishPage } from "./batch-publish";
export { XianyuBlacklistPage } from "./blacklist";
export { XianyuCardsPage } from "./cards";
export { XianyuDashboardPage } from "./dashboard";
export { XianyuDisclaimerPage } from "./disclaimer";
export { XianyuFeedbackPage } from "./feedback";
export { XianyuItemsPage } from "./items";
export { XianyuKeywordsPage } from "./keywords";
export { XianyuMessageFiltersPage } from "./message-filters";
export { XianyuMessageLogsPage } from "./message-logs";
export { XianyuMessageNotificationsPage } from "./message-notifications";
export { XianyuNotificationChannelsPage } from "./notification-channels";
export { XianyuOrdersPage } from "./orders";
export { XianyuPersonalSettingsPage } from "./personal-settings";
export { XianyuProductMaterialsPage } from "./product-materials";
export { XianyuProductPublishPage } from "./product-publish";
export { XianyuPublishAddressesPage } from "./publish-addresses";
export { XianyuPublishLogsPage } from "./publish-logs";
export { XianyuRiskLogsPage } from "./risk-logs";
export { XianyuTutorialPage } from "./tutorial";
export { XianyuManageConsole } from "./manage-console";
