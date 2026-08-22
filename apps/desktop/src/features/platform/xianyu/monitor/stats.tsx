import type { MonitorStats } from "@desk/platform/ipc/xianyu-monitor";

export function StatCard({
  label,
  value,
  highlight,
}: {
  label: string;
  value: number;
  highlight?: boolean;
}) {
  return (
    <div
      className={`rounded-xl border p-3 ${
        highlight ? "border-primary/40 bg-primary/5" : "border-border bg-card"
      }`}
    >
      <p className="text-[10px] uppercase tracking-wide text-muted-foreground">{label}</p>
      <p className="mt-0.5 text-xl font-semibold tabular-nums">{value}</p>
    </div>
  );
}

/** 顶部统计卡（任务 / 运行中 / 已启用 / 命中 / 推荐 / 今日新增）。 */
export function StatsSection({ stats }: { stats: MonitorStats }) {
  return (
    <div className="grid gap-3 sm:grid-cols-3 lg:grid-cols-6">
      <StatCard label="任务" value={stats.taskCount} />
      <StatCard label="运行中" value={stats.runningCount} highlight={stats.runningCount > 0} />
      <StatCard label="已启用" value={stats.enabledCount} />
      <StatCard label="累计命中" value={stats.resultCount} />
      <StatCard label="AI 推荐" value={stats.recommendedCount} />
      <StatCard label="今日新增" value={stats.todayNewCount} />
    </div>
  );
}
