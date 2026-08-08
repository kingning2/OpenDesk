/**
 * 首页数据看板状态 hook：加载聚合统计、loading / error、手动刷新。
 *
 * @author coisini
 * @created 2026-08-08
 */

import { useCallback, useEffect, useState } from "react";
import { dashboardStats, type DashboardStats } from "@desk/platform";

const EMPTY_STATS: DashboardStats = {
  total_channels: 0,
  total_emails: 0,
  total_verified_emails: 0,
  customer_total: 0,
  mail_total: 0,
  by_platform: [],
  by_email_status: [],
};

/**
 * 拉取首页看板聚合统计。
 *
 * @author coisini
 * @created 2026-08-08
 *
 * @returns 统计值、加载态、错误与刷新回调
 */
export function useHomeDashboard() {
  const [stats, setStats] = useState<DashboardStats>(EMPTY_STATS);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      setStats(await dashboardStats());
    } catch (cause) {
      setError(String(cause));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    const timer = window.setTimeout(() => {
      void refresh();
    }, 0);
    return () => {
      window.clearTimeout(timer);
    };
  }, [refresh]);

  return { stats, loading, error, refresh };
}
