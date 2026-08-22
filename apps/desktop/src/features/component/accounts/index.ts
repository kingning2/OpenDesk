/**
 * 账号管理共享层出口 — 供各平台薄入口复用。
 *
 * @author Xiaoman
 * @created 2026-08-22
 */

export type { AccountPanelDeps, AccountsTab, AutoConnectApi } from "./types";
export { AccountsPanel, resolveAccountPlatform } from "./accounts-panel";
export { AccountsHubPage } from "./accounts-hub";
export { AccountQrDialog } from "./account-qr-dialog";
