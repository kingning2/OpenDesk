/**
 * 闲鱼业务 feature — 原前端管理后台页面迁入。
 *
 * 迁移策略：
 * - 公共组件 / 工具 → `@desk/ui` / `@desk/utils`（已抽取）
 * - 数据访问 → Tauri IPC（`@desk/platform/ipc/*`，复用 crates/app Rust 业务）
 * - 页面 → 按原前端结构与交互风格重写，按业务模块拆分
 *
 * 精简说明：仅保留 账号管理 / 商品管理 / 订单管理；
 * 风控日志与免责声明已迁入应用设置弹窗。
 */

export { XianyuAccountsPage } from "./accounts";
export { XianyuDashboardPage } from "./dashboard";
export { XianyuItemsPage } from "./items";
export { XianyuOrdersPage } from "./orders";
export { XianyuManageConsole } from "./manage-console";
