/**
 * 1688 管理子页出口 — 仅账号管理一项，直接渲染共享账号面板。
 *
 * 1688 站点无闲鱼业务子页，管理根路径与 `/manage/accounts` 均落到账号管理。
 *
 * @author Xiaoman
 * @created 2026-08-22
 */

import { Ali1688AccountsPage } from "./accounts";

/**
 * 1688 管理子页出口（静态 URL → 业务页）。
 *
 * @author Xiaoman
 * @created 2026-08-22
 *
 * @returns 当前子页内容（1688 仅账号管理）
 */
export function Ali1688ManageConsole() {
  return <Ali1688AccountsPage />;
}
