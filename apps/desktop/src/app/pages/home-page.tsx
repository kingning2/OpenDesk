/**
 * 首页：数据看板，展示爬虫采集与客户、邮件聚合统计。
 *
 * @author coisini
 * @created 2026-07-20
 * @updated 2026-08-08
 */

import type { LucideIcon } from "@desk/ui/icons";
import {
  BadgeCheck,
  BarChart3,
  Database,
  Mail,
  MailPlus,
  RefreshCw,
  Users,
} from "@desk/ui/icons";
import {
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  Legend,
  Pie,
  PieChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import {
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  PageScaffold,
  cn,
} from "@desk/ui";

import { useT } from "../../i18n";
import { useHomeDashboard } from "./use-home-dashboard";

/** 图表系列调色板（与设计系统协调，明暗主题下均清晰）。 */
const CHART_COLORS = [
  "oklch(0.62 0.19 285)",
  "oklch(0.7 0.13 200)",
  "oklch(0.75 0.14 160)",
  "oklch(0.78 0.14 80)",
  "oklch(0.68 0.17 25)",
  "oklch(0.62 0.15 320)",
  "oklch(0.72 0.12 230)",
];

const TOOLTIP_STYLE = {
  borderRadius: "var(--radius-md)",
  border: "1px solid var(--color-border)",
  background: "var(--color-dialog)",
  color: "var(--color-dialog-foreground)",
  fontSize: "var(--text-sm)",
} as const;

/**
 * 应用首页。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @returns 首页节点
 */
export function HomePage() {
  const t = useT();
  const { stats, loading, error, refresh } = useHomeDashboard();

  const statCards: Array<{
    key: string;
    labelKey: string;
    icon: LucideIcon;
    value: number;
    accent?: boolean;
  }> = [
    { key: "channels", labelKey: "home.stat-channels", icon: Database, value: stats.total_channels },
    { key: "emails", labelKey: "home.stat-emails", icon: Mail, value: stats.total_emails, accent: true },
    { key: "verified", labelKey: "home.stat-verified-emails", icon: BadgeCheck, value: stats.total_verified_emails },
    { key: "customers", labelKey: "home.stat-customers", icon: Users, value: stats.customer_total },
    { key: "mails", labelKey: "home.stat-mails", icon: MailPlus, value: stats.mail_total },
  ];

  const platformData = stats.by_platform.map((bucket) => ({
    name: bucket.key.trim() || t("home.platform-other"),
    count: bucket.count,
  }));

  const statusData = stats.by_email_status.map((bucket) => ({
    name: t(`crawler-results.emailStatus.${bucket.key}`),
    value: bucket.count,
  }));

  return (
    <PageScaffold className="overflow-y-auto">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="text-(length:--text-lg) font-semibold">{t("home.dashboard-title")}</h1>
          <p className="text-(length:--text-sm) text-muted-foreground">
            {t("home.dashboard-subtitle")}
          </p>
        </div>
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={loading}
          onClick={() => void refresh()}
        >
          <RefreshCw className={cn("size-3.5", loading && "animate-spin")} />
          {t("home.refresh")}
        </Button>
      </div>

      {error ? (
        <p className="text-(length:--text-sm) text-destructive">{error}</p>
      ) : null}

      <div className="grid grid-cols-2 gap-3 md:grid-cols-3 xl:grid-cols-5">
        {statCards.map((card) => {
          const Icon = card.icon;
          return (
            <Card key={card.key} variant="glass" className="w-full">
              <CardHeader compact>
                <CardTitle className="flex items-center gap-2 text-(length:--text-xs) font-medium text-muted-foreground">
                  <Icon className="size-3.5" aria-hidden />
                  {t(card.labelKey)}
                </CardTitle>
              </CardHeader>
              <CardContent className="pt-2">
                <p
                  className={cn(
                    "font-display text-2xl font-semibold leading-none",
                    card.accent && "text-primary",
                  )}
                >
                  {card.value.toLocaleString()}
                </p>
              </CardContent>
            </Card>
          );
        })}
      </div>

      <div className="grid gap-3 lg:grid-cols-2">
        <Card variant="glass" className="w-full">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <BarChart3 className="size-4" aria-hidden />
              {t("home.chart-by-platform")}
            </CardTitle>
          </CardHeader>
          <CardContent>
            {loading ? (
              <ChartEmpty label={t("home.loading")} />
            ) : platformData.length === 0 ? (
              <ChartEmpty label={t("home.chart-platform-empty")} />
            ) : (
              <ResponsiveContainer width="100%" height={280}>
                <BarChart data={platformData} margin={{ top: 8, right: 8, left: -16, bottom: 0 }}>
                  <CartesianGrid strokeDasharray="3 3" stroke="var(--color-border)" vertical={false} />
                  <XAxis
                    dataKey="name"
                    tick={{ fill: "var(--color-muted-foreground)", fontSize: 12 }}
                    axisLine={{ stroke: "var(--color-border)" }}
                    tickLine={false}
                  />
                  <YAxis
                    allowDecimals={false}
                    tick={{ fill: "var(--color-muted-foreground)", fontSize: 12 }}
                    axisLine={false}
                    tickLine={false}
                  />
                  <Tooltip contentStyle={TOOLTIP_STYLE} cursor={{ fill: "var(--color-muted)" }} />
                  <Bar dataKey="count" fill="var(--color-primary)" radius={[6, 6, 0, 0]} />
                </BarChart>
              </ResponsiveContainer>
            )}
          </CardContent>
        </Card>

        <Card variant="glass" className="w-full">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Mail className="size-4" aria-hidden />
              {t("home.chart-by-email-status")}
            </CardTitle>
          </CardHeader>
          <CardContent>
            {loading ? (
              <ChartEmpty label={t("home.loading")} />
            ) : statusData.length === 0 ? (
              <ChartEmpty label={t("home.chart-status-empty")} />
            ) : (
              <ResponsiveContainer width="100%" height={280}>
                <PieChart>
                  <Pie
                    data={statusData}
                    dataKey="value"
                    nameKey="name"
                    cx="50%"
                    cy="50%"
                    innerRadius={56}
                    outerRadius={92}
                    paddingAngle={3}
                  >
                    {statusData.map((entry, index) => (
                      <Cell
                        key={entry.name}
                        fill={CHART_COLORS[index % CHART_COLORS.length]}
                        stroke="var(--color-workspace)"
                      />
                    ))}
                  </Pie>
                  <Tooltip contentStyle={TOOLTIP_STYLE} />
                  <Legend
                    iconType="circle"
                    iconSize={8}
                    formatter={(value) => (
                      <span style={{ color: "var(--color-muted-foreground)", fontSize: "var(--text-xs)" }}>
                        {value}
                      </span>
                    )}
                  />
                </PieChart>
              </ResponsiveContainer>
            )}
          </CardContent>
        </Card>
      </div>
    </PageScaffold>
  );
}

function ChartEmpty({ label }: { label: string }) {
  return (
    <div className="flex h-[280px] items-center justify-center text-(length:--text-sm) text-muted-foreground">
      {label}
    </div>
  );
}
