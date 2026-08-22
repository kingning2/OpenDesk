/**
 * 仪表盘 IPC 封装 — 平台业务统计聚合。
 *
 * 后端：壳层 `commands/dashboard.rs`（复用各业务 store 聚合统计）。
 *
 * @author agent
 * @created 2026-08-13
 */

import { call } from "./invoke";

/** 仪表盘统计快照（与 Rust `DashboardStats` 对齐）。 */
export interface DashboardStats {
  total_accounts: number;
  active_accounts: number;
  total_items: number;
  total_orders: number;
  pending_ship_orders: number;
}

/** 查询仪表盘统计。 */
export function dashboardStats(ownerId: number): Promise<DashboardStats> {
  return call<DashboardStats>("dashboard_stats", { ownerId });
}
