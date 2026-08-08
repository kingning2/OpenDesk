/**
 * Home dashboard aggregate stats IPC.
 *
 * @author coisini
 * @created 2026-08-08
 */

import type { DashboardIpcStatsRequest, DashboardIpcStatsResponse } from "@desk/contracts";

import { invokeIpc } from "./invoke";

/** One `GROUP BY` bucket: a category label plus its count. */
export interface DashboardCountBucket {
  key: string;
  count: number;
}

/** Parsed `stats_json` payload from `dashboard_stats`. */
export interface DashboardStats {
  total_channels: number;
  total_emails: number;
  total_verified_emails: number;
  customer_total: number;
  mail_total: number;
  by_platform: DashboardCountBucket[];
  by_email_status: DashboardCountBucket[];
}

/**
 * Fetch aggregate crawler/customer/mail stats for the home dashboard.
 *
 * @author coisini
 * @created 2026-08-08
 */
export async function dashboardStats(
  input: DashboardIpcStatsRequest = {},
): Promise<DashboardStats> {
  const response = await invokeIpc<DashboardIpcStatsResponse>("dashboard_stats", {
    request: input,
  });
  try {
    const parsed = JSON.parse(response.stats_json ?? "{}") as DashboardStats;
    return {
      total_channels: parsed.total_channels ?? 0,
      total_emails: parsed.total_emails ?? 0,
      total_verified_emails: parsed.total_verified_emails ?? 0,
      customer_total: parsed.customer_total ?? 0,
      mail_total: parsed.mail_total ?? 0,
      by_platform: Array.isArray(parsed.by_platform) ? parsed.by_platform : [],
      by_email_status: Array.isArray(parsed.by_email_status) ? parsed.by_email_status : [],
    };
  } catch {
    return {
      total_channels: 0,
      total_emails: 0,
      total_verified_emails: 0,
      customer_total: 0,
      mail_total: 0,
      by_platform: [],
      by_email_status: [],
    };
  }
}
