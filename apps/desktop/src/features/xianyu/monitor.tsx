/**
 * 闲鱼商品监控 — 任务 / 运行记录管理；点击某次运行进入详情页（agent 转录式）。
 */

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  AsyncButton,
  Button,
  Input,
  PageScaffold,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
  Textarea,
  toast,
} from "@desk/ui";
import {
  CheckCircle,
  ChevronDown,
  ChevronRight,
  ChevronUp,
  Loader2,
  Play,
  Trash2,
  XCircle,
} from "@desk/ui/icons";
import type { AiAccount, AiProvider } from "@desk/contracts";
import { managePath } from "@desk/platform/compile";
import { aiConfigGet } from "@desk/platform/ipc/ai";
import { accountList, type XianyuAccount } from "@desk/platform/ipc/account";
import { listenMonitorProgress } from "@desk/platform/events";
import {
  monitorGenerateKeywords,
  monitorRunList,
  monitorStats,
  monitorTaskDelete,
  monitorTaskList,
  monitorTaskRun,
  monitorTaskSave,
  type MonitorRun,
  type MonitorStats,
  type MonitorTask,
} from "@desk/platform/ipc/xianyu-monitor";
import { useWorkspaceNav } from "../../app/use-workspace-tabs";
import { formatRunTime } from "./monitor-console";
import { BUILT_IN_PROVIDERS } from "@feature/ai/builtin-providers";

const OWNER_ID = 1;

const EMPTY_FORM = {
  name: "",
  intent: "",
  keywords: "",
  accountId: "",
  aiAccountId: "",
  aiFailoverEnabled: true,
  aiAccountOrder: [] as string[],
  intervalMinutes: "5",
  aiCriteria: "",
  enabled: true,
};

interface AiAccountOption {
  id: string;
  label: string;
}

function toAccountOptions(accounts: XianyuAccount[]) {
  return accounts.map((account) => ({
    id: account.account_id,
    label: account.display_name || account.login_id || account.account_id,
  }));
}

function toAiAccountOptions(accounts: AiAccount[], providers: AiProvider[]): AiAccountOption[] {
  const providerName = new Map(providers.map((provider) => [provider.id, provider.name]));
  const options: AiAccountOption[] = accounts.map((account) => ({
    id: account.id,
    label: `${providerName.get(account.provider_id) ?? account.provider_id} · ${account.name}`,
  }));
  for (const provider of BUILT_IN_PROVIDERS) {
    if (provider.authless) {
      options.push({
        id: `provider:${provider.id}`,
        label: `${provider.name}（本地）`,
      });
    }
  }
  return options;
}

function sanitizeAiAccountOrder(order: string[], primaryId: string): string[] {
  const seen = new Set<string>();
  return order.filter((id) => {
    if (!id || id === primaryId || seen.has(id)) {
      return false;
    }
    seen.add(id);
    return true;
  });
}

function moveAiAccountOrder(order: string[], index: number, delta: -1 | 1): string[] {
  const nextIndex = index + delta;
  if (nextIndex < 0 || nextIndex >= order.length) {
    return order;
  }
  const next = [...order];
  const [item] = next.splice(index, 1);
  next.splice(nextIndex, 0, item!);
  return next;
}

function StatCard({
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

function RunRecordRow({ run, onOpen }: { run: MonitorRun; onOpen: () => void }) {
  const running = run.status === "running";
  return (
    <li>
      <button
        type="button"
        onClick={onOpen}
        className="flex w-full items-center gap-2 rounded-lg border border-border bg-card px-3 py-2 text-left transition-colors hover:bg-muted/40"
      >
        {running ? (
          <Loader2 className="size-4 shrink-0 animate-spin text-primary" />
        ) : run.status === "success" ? (
          <CheckCircle className="size-4 shrink-0 text-emerald-500" />
        ) : (
          <XCircle className="size-4 shrink-0 text-destructive" />
        )}
        <span className="min-w-0 flex-1 space-y-0.5">
          <span className="block text-xs font-medium">
            {formatRunTime(run.startedAt)}
            {running ? " · 运行中" : run.status === "success" ? " · 成功" : " · 失败"}
          </span>
          <span className="block text-[10px] text-muted-foreground">
            扫描 {run.scanned} · 新增 {run.newItems} · 推荐 {run.recommended}
            {run.error ? ` · ${run.error}` : ""}
          </span>
        </span>
        <ChevronRight className="size-4 shrink-0 text-muted-foreground" />
      </button>
    </li>
  );
}

export function XianyuMonitorPage() {
  const { selectTab } = useWorkspaceNav();
  const [accounts, setAccounts] = useState<XianyuAccount[]>([]);
  const [aiAccountOptions, setAiAccountOptions] = useState<AiAccountOption[]>([]);
  const [tasks, setTasks] = useState<MonitorTask[]>([]);
  const [selectedId, setSelectedId] = useState<string>("");
  const [runs, setRuns] = useState<MonitorRun[]>([]);
  const [runningTaskId, setRunningTaskId] = useState<string | null>(null);
  const [editing, setEditing] = useState(false);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [generating, setGenerating] = useState(false);
  const [form, setForm] = useState(EMPTY_FORM);
  const [stats, setStats] = useState<MonitorStats | null>(null);
  const pendingNavTaskId = useRef<string | null>(null);

  const accountOptions = useMemo(() => toAccountOptions(accounts), [accounts]);
  const aiAccountLabelMap = useMemo(
    () => new Map(aiAccountOptions.map((option) => [option.id, option.label])),
    [aiAccountOptions],
  );
  const addableAiAccounts = useMemo(
    () =>
      aiAccountOptions.filter(
        (option) =>
          option.id !== form.aiAccountId && !form.aiAccountOrder.includes(option.id),
      ),
    [aiAccountOptions, form.aiAccountId, form.aiAccountOrder],
  );
  const selectedTask = useMemo(
    () => tasks.find((task) => task.id === selectedId) ?? null,
    [tasks, selectedId],
  );

  const loadTasks = useCallback(async () => {
    const list = await monitorTaskList(OWNER_ID);
    setTasks(list);
    if (!selectedId && list.length > 0) {
      setSelectedId(list[0]!.id);
    }
  }, [selectedId]);

  const loadRuns = useCallback(async (taskId: string) => {
    if (!taskId) {
      setRuns([]);
      return [];
    }
    const list = await monitorRunList(OWNER_ID, taskId);
    setRuns(list);
    return list;
  }, []);

  const loadStats = useCallback(async () => {
    const value = await monitorStats(OWNER_ID);
    setStats(value);
  }, []);

  useEffect(() => {
    let cancelled = false;
    void aiConfigGet()
      .then((config) => {
        if (cancelled) return;
        const options = toAiAccountOptions(config.accounts, config.providers);
        setAiAccountOptions(options);
        if (options.length > 0) {
          setForm((current) => ({
            ...current,
            aiAccountId: current.aiAccountId || options[0]!.id,
          }));
        }
      })
      .catch((error) => {
        if (!cancelled) toast.error(error instanceof Error ? error.message : String(error));
      });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    let cancelled = false;
    void accountList(OWNER_ID)
      .then((list) => {
        if (cancelled) return;
        const xianyuAccounts = list.filter(
          (item) => (item.platform ?? "xianyu") === "xianyu" && item.status === "active",
        );
        setAccounts(xianyuAccounts);
        if (xianyuAccounts.length > 0) {
          setForm((current) => ({
            ...current,
            accountId: current.accountId || xianyuAccounts[0]!.account_id,
          }));
        }
      })
      .catch((error) => {
        if (!cancelled) toast.error(error instanceof Error ? error.message : String(error));
      });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    setLoading(true);
    void loadTasks()
      .catch((error) => toast.error(error instanceof Error ? error.message : String(error)))
      .finally(() => setLoading(false));
  }, [loadTasks]);

  useEffect(() => {
    void loadStats().catch((error) =>
      toast.error(error instanceof Error ? error.message : String(error)),
    );
  }, [loadStats]);

  useEffect(() => {
    if (!selectedId) {
      setRuns([]);
      return;
    }
    setRunningTaskId(null);
    void loadRuns(selectedId)
      .then((list) => {
        if (list[0]?.status === "running") {
          setRunningTaskId(selectedId);
        }
      })
      .catch((error) =>
        toast.error(error instanceof Error ? error.message : String(error)),
      );
  }, [selectedId, loadRuns]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listenMonitorProgress((payload) => {
      if (payload.taskId !== selectedId) return;
      if (payload.stage === "started") {
        if (pendingNavTaskId.current === payload.taskId) {
          pendingNavTaskId.current = null;
          setRunningTaskId(null);
          selectTab(`${managePath("monitor")}/runs/${payload.runId}`);
          return;
        }
        setRunningTaskId(payload.taskId);
        void loadRuns(payload.taskId);
        return;
      }
      if (payload.stage === "finished" || payload.stage === "failed") {
        setRunningTaskId(null);
        void loadRuns(payload.taskId);
      }
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, [selectedId, loadRuns, selectTab]);

  function resetForm(task?: MonitorTask) {
    if (task) {
      setForm({
        name: task.name,
        intent: task.intent,
        keywords: task.keywords.join("\n"),
        accountId: task.accountId,
        aiAccountId: task.aiAccountId || aiAccountOptions[0]?.id || "",
        aiFailoverEnabled: task.aiFailoverEnabled ?? true,
        aiAccountOrder: task.aiAccountOrder ?? [],
        intervalMinutes: String(task.intervalMinutes),
        aiCriteria: task.aiCriteria,
        enabled: task.enabled,
      });
      return;
    }
    setForm({
      ...EMPTY_FORM,
      accountId: accountOptions[0]?.id ?? "",
      aiAccountId: aiAccountOptions[0]?.id ?? "",
    });
  }

  async function handleSave() {
    if (!form.name.trim() || !form.intent.trim() || !form.aiCriteria.trim()) {
      toast.error("请填写任务名称、购买意图和 AI 筛选标准");
      return;
    }
    if (!form.accountId) {
      toast.error("请选择闲鱼账号");
      return;
    }
    if (!form.aiAccountId) {
      toast.error("请选择 AI 账号");
      return;
    }
    setSaving(true);
    try {
      const saved = await monitorTaskSave({
        ownerId: OWNER_ID,
        id: selectedTask?.id,
        name: form.name.trim(),
        intent: form.intent.trim(),
        keywords: form.keywords
          .split(/[\n,]+/)
          .map((item) => item.trim())
          .filter(Boolean),
        accountId: form.accountId,
        aiAccountId: form.aiAccountId,
        aiFailoverEnabled: form.aiFailoverEnabled,
        aiAccountOrder: sanitizeAiAccountOrder(form.aiAccountOrder, form.aiAccountId),
        intervalMinutes: Number(form.intervalMinutes) || 5,
        enabled: form.enabled,
        aiCriteria: form.aiCriteria.trim(),
      });
      toast.success("监控任务已保存");
      await loadTasks();
      setSelectedId(saved.id);
      setEditing(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

  async function handleGenerateKeywords() {
    if (!form.intent.trim() || !form.aiCriteria.trim()) {
      toast.error("请先填写购买意图和 AI 筛选标准");
      return;
    }
    if (!form.aiAccountId) {
      toast.error("请先选择 AI 账号");
      return;
    }
    setGenerating(true);
    try {
      const keywords = await monitorGenerateKeywords({
        ownerId: OWNER_ID,
        intent: form.intent.trim(),
        aiCriteria: form.aiCriteria.trim(),
        aiAccountId: form.aiAccountId,
        aiFailoverEnabled: form.aiFailoverEnabled,
        aiAccountOrder: sanitizeAiAccountOrder(form.aiAccountOrder, form.aiAccountId),
      });
      setForm((current) => ({ ...current, keywords: keywords.join("\n") }));
      toast.success(`已生成 ${keywords.length} 个关键词`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setGenerating(false);
    }
  }

  function handleRun(taskId: string) {
    pendingNavTaskId.current = taskId;
    setRunningTaskId(taskId);
    void monitorTaskRun(OWNER_ID, taskId).catch((error) => {
      toast.error(error instanceof Error ? error.message : String(error));
      pendingNavTaskId.current = null;
      setRunningTaskId(null);
    });
  }

  async function handleDelete(taskId: string) {
    try {
      await monitorTaskDelete(OWNER_ID, taskId);
      toast.success("任务已删除");
      if (selectedId === taskId) {
        setSelectedId("");
        resetForm();
        setEditing(true);
      }
      await loadTasks();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  return (
    <PageScaffold
      title="商品监控"
      subtitle="定时多任务并发：AI 生成关键词 → 列表搜索 → AI 决策 → 结果落库；点击某次运行进入详情页查看完整过程与商品。"
    >
      {stats ? (
        <div className="grid gap-3 sm:grid-cols-3 lg:grid-cols-6">
          <StatCard label="任务" value={stats.taskCount} />
          <StatCard
            label="运行中"
            value={stats.runningCount}
            highlight={stats.runningCount > 0}
          />
          <StatCard label="已启用" value={stats.enabledCount} />
          <StatCard label="累计命中" value={stats.resultCount} />
          <StatCard label="AI 推荐" value={stats.recommendedCount} />
          <StatCard label="今日新增" value={stats.todayNewCount} />
        </div>
      ) : null}

      <div className="grid gap-6 lg:grid-cols-[280px_minmax(0,1fr)]">
        <section className="space-y-3">
          <div className="flex items-center justify-between">
            <h2 className="text-sm font-medium">监控任务</h2>
            <Button
              size="sm"
              variant="outline"
              onClick={() => {
                setSelectedId("");
                resetForm();
                setEditing(true);
              }}
            >
              新建
            </Button>
          </div>
          <ul className="space-y-2">
            {tasks.map((task) => (
              <li key={task.id}>
                <button
                  type="button"
                  onClick={() => {
                    setSelectedId(task.id);
                    resetForm(task);
                    setEditing(false);
                  }}
                  className={`w-full rounded-lg border px-3 py-2 text-left text-sm transition-colors ${
                    selectedId === task.id
                      ? "border-primary bg-primary/10"
                      : "border-border bg-card hover:bg-muted/40"
                  }`}
                >
                  <div className="flex items-center justify-between gap-2">
                    <span className="font-medium">{task.name}</span>
                    {task.isRunning ? (
                      <span className="text-xs text-primary">运行中</span>
                    ) : task.enabled ? (
                      <span className="text-xs text-muted-foreground">启用</span>
                    ) : (
                      <span className="text-xs text-muted-foreground">停用</span>
                    )}
                  </div>
                  <p className="mt-1 line-clamp-2 text-xs text-muted-foreground">{task.intent}</p>
                </button>
              </li>
            ))}
          </ul>
          {!loading && tasks.length === 0 ? (
            <p className="text-sm text-muted-foreground">暂无监控任务，右侧创建第一个。</p>
          ) : null}
        </section>

        <section className="space-y-6">
          {selectedTask ? (
            <div className="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-border bg-card p-3">
              <div className="min-w-0 space-y-0.5">
                <h2 className="truncate text-sm font-medium">{selectedTask.name}</h2>
                <p className="text-xs text-muted-foreground">
                  {selectedTask.isRunning || runningTaskId === selectedTask.id
                    ? "运行中"
                    : selectedTask.enabled
                      ? "已启用"
                      : "已停用"}
                  {" · "}
                  间隔 {selectedTask.intervalMinutes} 分钟 · {selectedTask.keywords.length} 个关键词
                </p>
                {selectedTask.lastError ? (
                  <p className="text-xs text-destructive">上次错误：{selectedTask.lastError}</p>
                ) : null}
              </div>
              <div className="flex flex-wrap gap-2">
                <AsyncButton
                  onClick={() => handleRun(selectedTask.id)}
                  disabled={runningTaskId === selectedTask.id}
                >
                  <Play className="mr-1.5 size-4" />
                  立即运行
                </AsyncButton>
                <Button variant="outline" onClick={() => setEditing(true)}>
                  编辑任务
                </Button>
                <Button variant="outline" onClick={() => void handleDelete(selectedTask.id)}>
                  <Trash2 className="mr-1.5 size-4" />
                  删除
                </Button>
              </div>
            </div>
          ) : null}

          {selectedTask ? (
            <section className="space-y-3">
              <h2 className="text-sm font-medium">运行记录</h2>
              {runs.length === 0 ? (
                <p className="text-sm text-muted-foreground">
                  {runningTaskId
                    ? "正在执行…"
                    : "暂无运行记录。点击「立即运行」进入详情页查看实时过程，或从历史运行记录进入。"}
                </p>
              ) : (
                <ol className="space-y-2">
                  {runs.map((run) => (
                    <RunRecordRow
                      key={run.id}
                      run={run}
                      onOpen={() => selectTab(`${managePath("monitor")}/runs/${run.id}`)}
                    />
                  ))}
                </ol>
              )}
            </section>
          ) : null}

          {editing || !selectedTask ? (
            <div className="space-y-3 rounded-xl border border-border bg-card p-4">
              <div className="flex items-center justify-between">
                <h2 className="text-sm font-medium">{selectedTask ? "编辑任务" : "新建任务"}</h2>
                {selectedTask ? (
                  <Button size="sm" variant="ghost" onClick={() => setEditing(false)}>
                    收起
                  </Button>
                ) : null}
              </div>
              <div className="grid gap-3 sm:grid-cols-2">
                <div className="space-y-1.5 sm:col-span-2">
                  <label className="text-xs text-muted-foreground">任务名称</label>
                  <Input
                    value={form.name}
                    onChange={(event) => setForm({ ...form, name: event.target.value })}
                    placeholder="例如：MacBook Air M1 捡漏"
                  />
                </div>
                <div className="space-y-1.5 sm:col-span-2">
                  <label className="text-xs text-muted-foreground">购买意图（AI 生成关键词）</label>
                  <Textarea
                    value={form.intent}
                    onChange={(event) => setForm({ ...form, intent: event.target.value })}
                    placeholder="描述你想买什么、预算、成色要求等"
                    rows={3}
                  />
                </div>
                <div className="space-y-1.5 sm:col-span-2">
                  <label className="text-xs text-muted-foreground">AI 筛选标准（决策用）</label>
                  <Textarea
                    value={form.aiCriteria}
                    onChange={(event) => setForm({ ...form, aiCriteria: event.target.value })}
                    placeholder="例如：M1 芯片、16G 内存、价格低于 3500、个人闲置优先"
                    rows={3}
                  />
                </div>
                <div className="space-y-1.5">
                  <label className="text-xs text-muted-foreground">闲鱼账号</label>
                  <Select
                    value={form.accountId}
                    onValueChange={(value) => setForm({ ...form, accountId: value })}
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="选择账号" />
                    </SelectTrigger>
                    <SelectContent>
                      {accountOptions.map((account) => (
                        <SelectItem key={account.id} value={account.id}>
                          {account.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-1.5 sm:col-span-2">
                  <label className="text-xs text-muted-foreground">AI 账号（关键词 + 决策）</label>
                  <Select
                    value={form.aiAccountId}
                    onValueChange={(value) =>
                      setForm((current) => ({
                        ...current,
                        aiAccountId: value,
                        aiAccountOrder: sanitizeAiAccountOrder(current.aiAccountOrder, value),
                      }))
                    }
                  >
                    <SelectTrigger>
                      <SelectValue placeholder="选择 AI 账号" />
                    </SelectTrigger>
                    <SelectContent>
                      {aiAccountOptions.map((option) => (
                        <SelectItem key={option.id} value={option.id}>
                          {option.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  {aiAccountOptions.length === 0 ? (
                    <p className="text-xs text-muted-foreground">
                      请先在「AI 配置」中添加 DeepSeek / 豆包账号，或使用本地 Ollama。
                    </p>
                  ) : null}
                </div>
                <div className="flex items-center gap-2 sm:col-span-2">
                  <Switch
                    checked={form.aiFailoverEnabled}
                    onCheckedChange={(checked) =>
                      setForm({ ...form, aiFailoverEnabled: checked })
                    }
                  />
                  <span className="text-sm">余额不足时自动切换备用 AI</span>
                </div>
                {form.aiFailoverEnabled ? (
                  <div className="space-y-2 sm:col-span-2">
                    <label className="text-xs text-muted-foreground">
                      备用账号顺序（首选失败后依次尝试；留空则自动使用其余 AI 账号）
                    </label>
                    {form.aiAccountOrder.length > 0 ? (
                      <ul className="space-y-2">
                        {form.aiAccountOrder.map((id, index) => (
                          <li
                            key={id}
                            className="flex items-center gap-2 rounded-lg border border-border bg-muted/20 px-2 py-1.5"
                          >
                            <span className="min-w-0 flex-1 truncate text-sm">
                              {index + 1}. {aiAccountLabelMap.get(id) ?? id}
                            </span>
                            <Button
                              type="button"
                              size="sm"
                              variant="outline"
                              disabled={index === 0}
                              aria-label="上移"
                              onClick={() =>
                                setForm({
                                  ...form,
                                  aiAccountOrder: moveAiAccountOrder(form.aiAccountOrder, index, -1),
                                })
                              }
                            >
                              <ChevronUp className="size-4" />
                            </Button>
                            <Button
                              type="button"
                              size="sm"
                              variant="outline"
                              disabled={index === form.aiAccountOrder.length - 1}
                              aria-label="下移"
                              onClick={() =>
                                setForm({
                                  ...form,
                                  aiAccountOrder: moveAiAccountOrder(form.aiAccountOrder, index, 1),
                                })
                              }
                            >
                              <ChevronDown className="size-4" />
                            </Button>
                            <Button
                              type="button"
                              size="sm"
                              variant="outline"
                              aria-label="移除"
                              onClick={() =>
                                setForm({
                                  ...form,
                                  aiAccountOrder: form.aiAccountOrder.filter((item) => item !== id),
                                })
                              }
                            >
                              <Trash2 className="size-4" />
                            </Button>
                          </li>
                        ))}
                      </ul>
                    ) : (
                      <p className="text-xs text-muted-foreground">
                        未配置时将按 AI 配置中的账号自动尝试。
                      </p>
                    )}
                    {addableAiAccounts.length > 0 ? (
                      <Select
                        key={form.aiAccountOrder.join(",")}
                        onValueChange={(value) => {
                          setForm({
                            ...form,
                            aiAccountOrder: [...form.aiAccountOrder, value],
                          });
                        }}
                      >
                        <SelectTrigger className="max-w-xs">
                          <SelectValue placeholder="添加备用账号" />
                        </SelectTrigger>
                        <SelectContent>
                          {addableAiAccounts.map((option) => (
                            <SelectItem key={option.id} value={option.id}>
                              {option.label}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    ) : null}
                  </div>
                ) : null}
                <div className="space-y-1.5">
                  <label className="text-xs text-muted-foreground">定时间隔（分钟）</label>
                  <Input
                    type="number"
                    min={1}
                    value={form.intervalMinutes}
                    onChange={(event) =>
                      setForm({ ...form, intervalMinutes: event.target.value })
                    }
                  />
                </div>
                <div className="space-y-1.5 sm:col-span-2">
                  <div className="flex items-center justify-between gap-2">
                    <label className="text-xs text-muted-foreground">
                      搜索关键词（可 AI 生成）
                    </label>
                    <Button
                      size="sm"
                      variant="outline"
                      disabled={generating}
                      onClick={() => void handleGenerateKeywords()}
                    >
                      {generating ? "生成中…" : "AI 生成关键词"}
                    </Button>
                  </div>
                  <Textarea
                    value={form.keywords}
                    onChange={(event) => setForm({ ...form, keywords: event.target.value })}
                    placeholder="每行一个关键词；留空则运行时自动生成"
                    rows={3}
                  />
                </div>
                <div className="flex items-center gap-2 sm:col-span-2">
                  <Switch
                    checked={form.enabled}
                    onCheckedChange={(checked) => setForm({ ...form, enabled: checked })}
                  />
                  <span className="text-sm">启用定时监控</span>
                </div>
              </div>
              <div className="flex flex-wrap gap-2">
                <AsyncButton loading={saving} onClick={() => void handleSave()}>
                  保存任务
                </AsyncButton>
              </div>
            </div>
          ) : null}
        </section>
      </div>
    </PageScaffold>
  );
}
