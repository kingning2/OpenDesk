/**
 * 闲鱼风控日志页（迁移自原前端 `pages/admin/RiskLogs.tsx`）。
 *
 * 按原前端核心交互重写：账号/日期/处理状态/调用类型/调用用户筛选 + 当日成功率 +
 * 远程过滑块配置（折叠面板）+ 清空/清空处理中。
 * 数据访问走 Tauri IPC（`@desk/platform/ipc/risk`），复用 crates/app RiskService。
 *
 * 说明：原前端的 is_admin 判断在桌面单用户场景下恒为当前用户，故直接展示全部操作。
 */

import { useEffect, useState } from "react";
import {
  Button,
  ConfirmModal,
  Input,
  Loading,
  PageScaffold,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  toast,
} from "@desk/ui";
import { Calendar, ChevronDown, ChevronUp, RefreshCw, Settings, ShieldAlert, Trash2, TrendingUp } from "@desk/ui/icons";
import { accountList, type XianyuAccount } from "@desk/platform/ipc/account";
import {
  riskConfigGet,
  riskConfigSet,
  riskLogClear,
  riskLogClearProcessing,
  riskLogList,
  riskLogTodayRate,
  type RiskConfig,
  type RiskLogItem,
  type RiskTodaySuccessRate,
} from "@desk/platform/ipc/risk";

const OWNER_ID = 1; // 桌面单用户；多用户时由登录态注入
const PAGE_SIZE_OPTIONS = [10, 20, 50, 100];
const TOKEN_API_ONLY_DOMAINS = ["api.xianyusite.shop", "api.zhinianblog.cn"];

const STATUS_LABELS: Record<string, string> = {
  success: "成功",
  failed: "失败",
  processing: "处理中",
  cancelled: "已取消",
};

const ENGINE_LABELS: Record<string, string> = {
  drissionpage: "兜底引擎",
  playwright: "主引擎",
  real_mouse: "真人鼠标",
  remote: "远程接口",
};

/** 前端筛选项。 */
interface Filters {
  accountId: string;
  startDate: string;
  endDate: string;
  status: string;
  callType: string;
  callUser: string;
}

const EMPTY_FILTERS: Filters = {
  accountId: "",
  startDate: "",
  endDate: "",
  status: "",
  callType: "",
  callUser: "",
};

/** 配置表单（字符串字段便于输入框编辑）。 */
interface ConfigForm {
  remoteUrl: string;
  remoteSecret: string;
  passCookies: boolean;
  blockRemoteCalls: boolean;
  localWeight: string;
  remoteWeight: string;
  remoteProcessingMax: string;
  remoteCooldownSeconds: string;
}

function toConfigForm(config: RiskConfig): ConfigForm {
  return {
    remoteUrl: config.remote_url,
    remoteSecret: config.remote_secret,
    passCookies: config.pass_cookies,
    blockRemoteCalls: config.block_remote_calls,
    localWeight: String(config.local_weight),
    remoteWeight: String(config.remote_weight),
    remoteProcessingMax: String(config.remote_processing_max),
    remoteCooldownSeconds: String(config.remote_cooldown_seconds),
  };
}

/**
 * 闲鱼风控日志页。
 *
 * @author agent
 * @created 2026-08-13
 */
export function XianyuRiskLogsPage() {
  const [logs, setLogs] = useState<RiskLogItem[]>([]);
  const [accounts, setAccounts] = useState<XianyuAccount[]>([]);
  const [filters, setFilters] = useState<Filters>(EMPTY_FILTERS);
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [total, setTotal] = useState(0);
  const [totalPages, setTotalPages] = useState(0);
  const [loading, setLoading] = useState(true);

  const [todayRate, setTodayRate] = useState<RiskTodaySuccessRate | null>(null);
  const [configExpanded, setConfigExpanded] = useState(false);
  const [config, setConfig] = useState<ConfigForm | null>(null);
  const [savingConfig, setSavingConfig] = useState(false);
  const [clearConfirm, setClearConfirm] = useState(false);
  const [clearProcessingConfirm, setClearProcessingConfirm] = useState(false);
  const [clearing, setClearing] = useState(false);

  async function load(nextPage = page, nextPageSize = pageSize) {
    setLoading(true);
    try {
      const result = await riskLogList({
        page: nextPage,
        page_size: nextPageSize,
        account_id: filters.accountId || undefined,
        start_date: filters.startDate || undefined,
        end_date: filters.endDate || undefined,
        processing_status: filters.status || undefined,
        call_type: filters.callType || undefined,
        call_user: filters.callUser.trim() || undefined,
      });
      setLogs(result.data);
      setPage(result.page);
      setPageSize(result.page_size);
      setTotal(result.total);
      setTotalPages(result.total_pages);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setLoading(false);
    }
  }

  async function loadTodayRate() {
    try {
      const rate = await riskLogTodayRate(new Date().toISOString().slice(0, 10));
      setTodayRate(rate);
    } catch {
      // 成功率加载失败不阻断页面
    }
  }

  useEffect(() => {
    let cancelled = false;
    void accountList(OWNER_ID)
      .then((list) => {
        if (!cancelled) setAccounts(list);
      })
      .catch(() => {
        // 账号列表加载失败不阻塞日志查询
      });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    let cancelled = false;
    void riskLogList({ page: 1, page_size: 20 })
      .then((result) => {
        if (cancelled) return;
        setLogs(result.data);
        setPage(result.page);
        setPageSize(result.page_size);
        setTotal(result.total);
        setTotalPages(result.total_pages);
      })
      .catch((error) => {
        if (!cancelled) {
          toast.error(error instanceof Error ? error.message : String(error));
        }
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    void riskLogTodayRate(new Date().toISOString().slice(0, 10))
      .then((rate) => {
        if (!cancelled) setTodayRate(rate);
      })
      .catch(() => {
        // 成功率加载失败不阻断页面
      });
    void riskConfigGet()
      .then((config) => {
        if (!cancelled) setConfig(toConfigForm(config));
      })
      .catch(() => {
        // 配置加载失败不阻断页面
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const patchFilters = (patch: Partial<Filters>) =>
    setFilters((current) => ({ ...current, ...patch }));

  function statusLabel(log: RiskLogItem): string {
    if (log.processing_status === "processing") {
      const call = log.call_type === "remote" ? "远程" : "本机";
      return `处理中（${call}）`;
    }
    return STATUS_LABELS[log.processing_status] ?? (log.processing_status || "-");
  }

  async function handleClear() {
    setClearing(true);
    try {
      await riskLogClear();
      toast.success("日志已清空");
      setClearConfirm(false);
      await load(1);
      await loadTodayRate();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setClearing(false);
    }
  }

  async function handleClearProcessing() {
    setClearing(true);
    try {
      await riskLogClearProcessing();
      toast.success("处理中日志已清空");
      setClearProcessingConfirm(false);
      await load(1);
      await loadTodayRate();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setClearing(false);
    }
  }

  const findTokenApiDomain = (url: string): string => {
    const lowered = url.trim().toLowerCase();
    return TOKEN_API_ONLY_DOMAINS.find((domain) => lowered.includes(domain)) ?? "";
  };

  async function handleSaveConfig() {
    if (!config) return;
    const tokenDomain = findTokenApiDomain(config.remoteUrl);
    if (tokenDomain) {
      toast.error(`该URL（${tokenDomain}）不是在此处填写，需要在「系统设置-Token获取方式」中填写`);
      return;
    }
    const parseNonnegative = (value: string, label: string): number | null => {
      const normalized = value.trim();
      if (!/^\d+$/.test(normalized)) {
        toast.error(`${label}必须是大于或等于 0 的整数`);
        return null;
      }
      return Number(normalized);
    };
    const processingMax = parseNonnegative(config.remoteProcessingMax, "远程处理中最大条数");
    if (processingMax === null) return;
    const cooldownSeconds = parseNonnegative(config.remoteCooldownSeconds, "远程调用冷却时间");
    if (cooldownSeconds === null) return;
    const normWeight = (value: string): number => {
      const normalized = value.trim();
      if (!normalized) return 1;
      const n = Number(normalized);
      return Number.isFinite(n) && n >= 0 ? n : 1;
    };
    setSavingConfig(true);
    try {
      await riskConfigSet({
        remote_url: config.remoteUrl.trim(),
        remote_secret: config.remoteSecret.trim(),
        pass_cookies: config.passCookies,
        block_remote_calls: config.blockRemoteCalls,
        local_weight: normWeight(config.localWeight),
        remote_weight: normWeight(config.remoteWeight),
        remote_processing_max: processingMax,
        remote_cooldown_seconds: cooldownSeconds,
        local_slider_disabled: false,
      });
      toast.success("远程过滑块配置已保存");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSavingConfig(false);
    }
  }

  const patchConfig = (patch: Partial<ConfigForm>) =>
    setConfig((current) => (current ? { ...current, ...patch } : current));

  return (
    <PageScaffold subtitle="闲鱼风控日志 — 滑块验证与风控事件记录">
      <div className="space-y-4">
        {/* 提示条 */}
        <div className="rounded-lg border border-red-200 bg-red-500/10 px-4 py-3">
          <p className="text-[length:var(--text-sm)] font-medium text-red-600">
            如遇滑块验证无法通过或者账号管理在线状态一直离线的场景，可到「系统设置 - 基础设置 - Token获取方式」，调整为远程接口。
          </p>
        </div>

        {/* 远程过滑块配置（折叠面板） */}
        <div className="rounded-xl border border-border bg-shell p-4">
          <div className="flex flex-wrap items-center gap-4">
            <button
              type="button"
              onClick={() => setConfigExpanded((value) => !value)}
              aria-expanded={configExpanded}
              className="flex items-center gap-2 text-[length:var(--text-sm)] font-medium transition-colors hover:text-primary"
            >
              <Settings className="size-4 text-primary" aria-hidden />
              远程过滑块配置
              {configExpanded ? (
                <ChevronUp className="size-4 opacity-50" aria-hidden />
              ) : (
                <ChevronDown className="size-4 opacity-50" aria-hidden />
              )}
              <span className="text-[length:var(--text-xs)] font-normal text-muted-foreground">
                {configExpanded ? "收起" : "展开"}
              </span>
            </button>
          </div>

          {configExpanded && config ? (
            <div className="mt-3 space-y-4 border-t border-border pt-3">
              <div className="flex flex-wrap items-end gap-3">
                <label className="block min-w-64 flex-1 space-y-1">
                  <span className="text-[length:var(--text-xs)] text-muted-foreground">远程服务URL</span>
                  <Input
                    value={config.remoteUrl}
                    onChange={(event) => patchConfig({ remoteUrl: event.target.value })}
                    placeholder="例如：https://your-host/api/v1/captcha/slider-solve"
                  />
                </label>
                <label className="block min-w-64 flex-1 space-y-1">
                  <span className="text-[length:var(--text-xs)] text-muted-foreground">秘钥</span>
                  <Input
                    value={config.remoteSecret}
                    onChange={(event) => patchConfig({ remoteSecret: event.target.value })}
                    placeholder="个人设置中的秘钥"
                  />
                </label>
                <Button onClick={() => void handleSaveConfig()} disabled={savingConfig}>
                  {savingConfig ? "保存中…" : "保存"}
                </Button>
              </div>

              <div className="flex flex-wrap gap-x-8 gap-y-4">
                <label className="flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={config.passCookies}
                    onChange={(event) => patchConfig({ passCookies: event.target.checked })}
                    className="size-4 accent-primary"
                  />
                  <span className="text-[length:var(--text-sm)]">调用远程接口时传递账号 Cookie</span>
                </label>
                <label className="flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={config.blockRemoteCalls}
                    onChange={(event) => patchConfig({ blockRemoteCalls: event.target.checked })}
                    className="size-4 accent-primary"
                  />
                  <span className="text-[length:var(--text-sm)]">禁止远程调用本机过滑块接口</span>
                </label>
              </div>

              <div className="flex flex-wrap items-end gap-3 border-t border-border pt-3">
                <label className="block w-44 space-y-1">
                  <span className="text-[length:var(--text-xs)] text-muted-foreground">远程处理中最大条数</span>
                  <Input
                    type="number"
                    min={0}
                    value={config.remoteProcessingMax}
                    onChange={(event) => patchConfig({ remoteProcessingMax: event.target.value })}
                  />
                </label>
                <label className="block w-44 space-y-1">
                  <span className="text-[length:var(--text-xs)] text-muted-foreground">远程调用冷却时间（秒）</span>
                  <Input
                    type="number"
                    min={0}
                    value={config.remoteCooldownSeconds}
                    onChange={(event) => patchConfig({ remoteCooldownSeconds: event.target.value })}
                  />
                </label>
                <label className="block w-32 space-y-1">
                  <span className="text-[length:var(--text-xs)] text-muted-foreground">本地排队权重</span>
                  <Input
                    type="number"
                    min={0}
                    value={config.localWeight}
                    onChange={(event) => patchConfig({ localWeight: event.target.value })}
                  />
                </label>
                <label className="block w-32 space-y-1">
                  <span className="text-[length:var(--text-xs)] text-muted-foreground">远程排队权重</span>
                  <Input
                    type="number"
                    min={0}
                    value={config.remoteWeight}
                    onChange={(event) => patchConfig({ remoteWeight: event.target.value })}
                  />
                </label>
              </div>
            </div>
          ) : null}
        </div>

        {/* 工具栏 */}
        <div className="flex items-center justify-between gap-3">
          <div className="flex gap-2">
            <Button
              size="sm"
              variant="outline"
              onClick={() => {
                void load(1);
                void loadTodayRate();
              }}
            >
              <RefreshCw className="size-4" aria-hidden />
              刷新
            </Button>
            <Button
              size="sm"
              variant="outline"
              className="text-destructive"
              onClick={() => setClearProcessingConfirm(true)}
            >
              <Trash2 className="size-4" aria-hidden />
              清空处理中
            </Button>
            <Button
              size="sm"
              variant="outline"
              className="text-destructive"
              onClick={() => setClearConfirm(true)}
            >
              <Trash2 className="size-4" aria-hidden />
              清空日志
            </Button>
          </div>
        </div>

        {/* 筛选区 */}
        <div className="rounded-xl border border-border bg-shell p-4">
          <div className="flex flex-wrap items-end gap-3">
            <label className="block space-y-1">
              <span className="text-[length:var(--text-xs)] text-muted-foreground">开始日期</span>
              <Input
                type="date"
                value={filters.startDate}
                onChange={(event) => patchFilters({ startDate: event.target.value })}
                className="w-40"
              />
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-xs)] text-muted-foreground">结束日期</span>
              <Input
                type="date"
                value={filters.endDate}
                onChange={(event) => patchFilters({ endDate: event.target.value })}
                className="w-40"
              />
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-xs)] text-muted-foreground">处理状态</span>
              <Select value={filters.status} onValueChange={(value) => patchFilters({ status: value })}>
                <SelectTrigger className="w-32" aria-label="处理状态">
                  <SelectValue placeholder="全部状态" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="">全部状态</SelectItem>
                  <SelectItem value="success">成功</SelectItem>
                  <SelectItem value="failed">失败</SelectItem>
                  <SelectItem value="processing">处理中</SelectItem>
                  <SelectItem value="cancelled">已取消</SelectItem>
                </SelectContent>
              </Select>
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-xs)] text-muted-foreground">调用类型</span>
              <Select value={filters.callType} onValueChange={(value) => patchFilters({ callType: value })}>
                <SelectTrigger className="w-32" aria-label="调用类型">
                  <SelectValue placeholder="全部类型" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="">全部类型</SelectItem>
                  <SelectItem value="local">本机</SelectItem>
                  <SelectItem value="remote">远程</SelectItem>
                </SelectContent>
              </Select>
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-xs)] text-muted-foreground">调用用户</span>
              <Input
                value={filters.callUser}
                onChange={(event) => patchFilters({ callUser: event.target.value })}
                onKeyDown={(event) => {
                  if (event.key === "Enter") void load(1);
                }}
                placeholder="模糊匹配，仅远程调用"
                className="w-44"
              />
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-xs)] text-muted-foreground">筛选账号</span>
              <Select
                value={filters.accountId}
                onValueChange={(value) => patchFilters({ accountId: value })}
              >
                <SelectTrigger className="w-44" aria-label="筛选账号">
                  <SelectValue placeholder="全部账号" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="">全部账号</SelectItem>
                  {accounts.map((account) => (
                    <SelectItem key={account.account_id} value={account.account_id}>
                      {account.remark
                        ? `${account.account_id} (${account.remark})`
                        : account.account_id}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </label>
            <Button onClick={() => void load(1)}>
              <Calendar className="size-4" aria-hidden />
              查询
            </Button>
          </div>
        </div>

        {/* 当日成功率 */}
        <div className="rounded-xl border border-border bg-shell px-4 py-3">
          <div className="flex flex-wrap items-center gap-x-6 gap-y-2">
            <div className="flex items-center gap-2 text-[length:var(--text-sm)] text-muted-foreground">
              <TrendingUp className="size-4 text-emerald-600" aria-hidden />
              <span>当日成功率{todayRate?.date ? `（${todayRate.date}）` : ""}</span>
            </div>
            <RateItem label="总体" value={todayRate ? `${todayRate.rate}%` : "-"} detail={todayRate ? `(${todayRate.success}/${todayRate.total})` : ""} />
            <RateItem label="本机" value={todayRate ? `${todayRate.local_rate}%` : "-"} detail={todayRate ? `(${todayRate.local_success}/${todayRate.local_total})` : ""} color="text-primary" />
            <RateItem label="远程" value={todayRate ? `${todayRate.remote_rate}%` : "-"} detail={todayRate ? `(${todayRate.remote_success}/${todayRate.remote_total})` : ""} color="text-orange-600" />
            <RateItem label="处理中总计" value={todayRate ? String(todayRate.processing) : "-"} color="text-amber-600" />
            <RateItem label="本机处理中" value={todayRate ? String(todayRate.local_processing) : "-"} color="text-primary" />
            <RateItem label="远程处理中" value={todayRate ? String(todayRate.remote_processing) : "-"} color="text-orange-600" />
          </div>
        </div>

        {/* 日志列表 */}
        {loading ? (
          <Loading size="lg" text="加载中..." className="py-16" />
        ) : logs.length === 0 ? (
          <div className="py-16 text-center text-muted-foreground">
            <ShieldAlert className="mx-auto mb-2 size-10 opacity-40" aria-hidden />
            暂无风控日志
          </div>
        ) : (
          <div className="overflow-hidden rounded-xl border border-border">
            <div className="flex items-center justify-between border-b border-border bg-muted/30 px-4 py-2">
              <span className="text-[length:var(--text-sm)] font-medium">风控日志</span>
              <span className="rounded-full bg-primary/15 px-2 py-0.5 text-[length:var(--text-xs)] text-primary">
                {total} 条记录
              </span>
            </div>
            <div className="max-h-[60vh] overflow-auto">
              <table className="w-full text-[length:var(--text-xs)]">
                <thead className="sticky top-0 bg-muted/80 text-muted-foreground">
                  <tr>
                    <th className="px-3 py-2 text-left font-medium">账号ID</th>
                    <th className="px-3 py-2 text-left font-medium">事件描述</th>
                    <th className="px-3 py-2 text-left font-medium">处理结果</th>
                    <th className="px-3 py-2 text-left font-medium">失败原因</th>
                    <th className="px-3 py-2 text-left font-medium">处理状态</th>
                    <th className="px-3 py-2 text-left font-medium">验证引擎</th>
                    <th className="px-3 py-2 text-left font-medium">调用类型</th>
                    <th className="px-3 py-2 text-left font-medium">调用用户</th>
                    <th className="px-3 py-2 text-left font-medium">创建时间</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border">
                  {logs.map((log) => (
                    <tr key={log.id} className="hover:bg-muted/30">
                      <td className="px-3 py-2 align-top font-medium text-primary">
                        {accounts.find((account) => account.account_id === log.account_id)?.remark
                          ? `${log.account_id} (${accounts.find((account) => account.account_id === log.account_id)?.remark})`
                          : log.account_id}
                      </td>
                      <td className="max-w-48 px-3 py-2 align-top text-muted-foreground" title={log.message}>
                        <span className="block truncate">{log.message || "-"}</span>
                      </td>
                      <td className="max-w-48 px-3 py-2 align-top text-muted-foreground" title={log.processing_result}>
                        <span className="block truncate">{log.processing_result || "-"}</span>
                      </td>
                      <td className="max-w-48 px-3 py-2 align-top text-destructive" title={log.error_message ?? ""}>
                        <span className="block truncate">{log.error_message || "-"}</span>
                      </td>
                      <td className="px-3 py-2 align-top">
                        <StatusBadge status={log.processing_status} label={statusLabel(log)} />
                      </td>
                      <td className="px-3 py-2 align-top">
                        {log.captcha_engine ? (
                          <span className="rounded-full bg-muted px-2 py-0.5 text-[length:var(--text-xs)]">
                            {ENGINE_LABELS[log.captcha_engine] ?? log.captcha_engine}
                          </span>
                        ) : (
                          "-"
                        )}
                      </td>
                      <td className="px-3 py-2 align-top">
                        <span
                          className={
                            log.call_type === "remote"
                              ? "rounded-full bg-orange-500/15 px-2 py-0.5 text-[length:var(--text-xs)] text-orange-600"
                              : "rounded-full bg-muted px-2 py-0.5 text-[length:var(--text-xs)] text-muted-foreground"
                          }
                        >
                          {log.call_type === "remote" ? "远程" : "本机"}
                        </span>
                      </td>
                      <td className="whitespace-nowrap px-3 py-2 align-top text-muted-foreground">
                        {log.call_user ?? "-"}
                      </td>
                      <td className="whitespace-nowrap px-3 py-2 align-top text-muted-foreground">
                        {log.created_at ?? "-"}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}

        {/* 分页 */}
        {total > 0 ? (
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="flex items-center gap-2 text-[length:var(--text-sm)] text-muted-foreground">
              <span>每页</span>
              <Select value={String(pageSize)} onValueChange={(value) => void load(1, Number(value))}>
                <SelectTrigger className="w-20" aria-label="每页条数">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {PAGE_SIZE_OPTIONS.map((size) => (
                    <SelectItem key={size} value={String(size)}>
                      {size}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <span>条，共 {total} 条</span>
            </div>
            <div className="flex items-center gap-2">
              <Button size="sm" variant="outline" disabled={page <= 1} onClick={() => void load(Math.max(1, page - 1))}>
                上一页
              </Button>
              <span className="text-[length:var(--text-sm)] text-muted-foreground">
                第 {page} / {Math.max(1, totalPages)} 页
              </span>
              <Button
                size="sm"
                variant="outline"
                disabled={page >= totalPages}
                onClick={() => void load(Math.min(totalPages, page + 1))}
              >
                下一页
              </Button>
            </div>
          </div>
        ) : null}
      </div>

      {/* 清空确认 */}
      <ConfirmModal
        isOpen={clearConfirm}
        type="danger"
        title="清空确认"
        message="确定要清空所有风控日志吗？此操作不可恢复！"
        confirmText="清空"
        loading={clearing}
        onConfirm={() => void handleClear()}
        onCancel={() => setClearConfirm(false)}
      />

      {/* 清空处理中确认 */}
      <ConfirmModal
        isOpen={clearProcessingConfirm}
        type="danger"
        title="清空处理中日志确认"
        message="确定要清空所有处理中状态的风控日志吗？此操作不可恢复！"
        confirmText="清空"
        loading={clearing}
        onConfirm={() => void handleClearProcessing()}
        onCancel={() => setClearProcessingConfirm(false)}
      />
    </PageScaffold>
  );
}

/** 成功率单项展示。 */
function RateItem({
  label,
  value,
  detail,
  color = "text-emerald-600",
}: {
  label: string;
  value: string;
  detail?: string;
  color?: string;
}) {
  return (
    <div className="flex items-baseline gap-1.5">
      <span className="text-[length:var(--text-sm)] text-muted-foreground">{label}</span>
      <span className={`text-lg font-bold ${color}`}>{value}</span>
      {detail ? <span className="text-[length:var(--text-xs)] text-muted-foreground">{detail}</span> : null}
    </div>
  );
}

/** 处理状态徽标。 */
function StatusBadge({ status, label }: { status: string; label: string }) {
  const color =
    status === "success"
      ? "bg-emerald-500/15 text-emerald-600"
      : status === "failed"
        ? "bg-red-500/15 text-red-600"
        : status === "processing"
          ? "bg-amber-500/15 text-amber-600"
          : "bg-muted text-muted-foreground";
  return (
    <span className={`rounded-full px-2 py-0.5 text-[length:var(--text-xs)] ${color}`}>{label}</span>
  );
}
