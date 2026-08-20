/**
 * 闲鱼仪表盘页（迁移自原前端 `pages/dashboard/Dashboard.tsx`）。
 *
 * 按原前端核心交互重写：统计卡片区（账号/关键词/商品/卡券/订单/待发货）。
 * 数据访问走 Tauri IPC（`@desk/platform/ipc/dashboard`），复用壳层聚合统计。
 */

import { useEffect, useState } from "react";
import { Activity, MessageSquare, Package, ShoppingCart, Ticket, Users } from "@desk/ui/icons";
import { Loading, PageScaffold } from "@desk/ui";
import { dashboardStats, type DashboardStats } from "@desk/platform/ipc/dashboard";

const OWNER_ID = 1; // 桌面单用户；多用户时由登录态注入

interface StatCardDef {
  key: keyof DashboardStats;
  icon: typeof Users;
  label: string;
  color: string;
}

const STAT_CARDS: StatCardDef[] = [
  { key: "total_accounts", icon: Users, label: "总账号数", color: "text-blue-500" },
  { key: "active_accounts", icon: Activity, label: "启用账号", color: "text-amber-500" },
  { key: "total_keywords", icon: MessageSquare, label: "关键词数", color: "text-emerald-500" },
  { key: "total_items", icon: Package, label: "商品数", color: "text-violet-500" },
  { key: "total_cards", icon: Ticket, label: "卡券数", color: "text-cyan-500" },
  { key: "total_orders", icon: ShoppingCart, label: "总订单", color: "text-blue-500" },
  { key: "pending_ship_orders", icon: ShoppingCart, label: "待发货", color: "text-amber-500" },
];

/**
 * 闲鱼仪表盘页。
 *
 * @author agent
 * @created 2026-08-13
 */
export function XianyuDashboardPage() {
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    void dashboardStats(OWNER_ID)
      .then((data) => {
        if (!cancelled) {
          setStats(data);
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <PageScaffold subtitle="闲鱼账号自动化运营概览">
      <header>
        <h1 className="text-[length:var(--text-xl)] font-semibold tracking-tight text-foreground">
          仪表盘
        </h1>
      </header>

      {loading ? (
        <Loading size="lg" text="加载中..." className="py-20" />
      ) : stats ? (
        <div className="mt-6 grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-4">
          {STAT_CARDS.map((card) => {
            const Icon = card.icon;
            return (
              <div
                key={card.key}
                className="rounded-[var(--radius-xl)] border border-border/70 bg-card p-5"
              >
                <div className="flex items-center gap-3">
                  <span className="flex size-11 shrink-0 items-center justify-center rounded-[var(--radius-lg)] bg-muted">
                    <Icon className={`size-5 ${card.color}`} aria-hidden />
                  </span>
                  <div className="min-w-0">
                    <p className="text-[length:var(--text-xs)] text-muted-foreground">
                      {card.label}
                    </p>
                    <p className="text-[length:var(--text-2xl)] font-semibold tabular-nums text-foreground">
                      {stats[card.key]}
                    </p>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      ) : null}
    </PageScaffold>
  );
}
