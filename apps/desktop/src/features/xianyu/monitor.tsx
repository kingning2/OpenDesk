/**
 * 闲鱼商品监控 — 多任务定时 + AI 关键词 / 决策 + SQLite 结果。
 */

import { useCallback, useEffect, useMemo, useState } from "react";
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
import { ChevronDown, ChevronUp, ExternalLink, Play, Trash2 } from "@desk/ui/icons";
import type { AiAccount, AiProvider } from "@desk/contracts";
import { aiConfigGet } from "@desk/platform/ipc/ai";
import { accountList, type XianyuAccount } from "@desk/platform/ipc/account";
import { listenMonitorMatch } from "@desk/platform/events";
import {
  monitorGenerateKeywords,
  monitorResultList,
  monitorTaskDelete,
  monitorTaskList,
  monitorTaskRun,
  monitorTaskSave,
  type MonitorResult,
  type MonitorTask,
} from "@desk/platform/ipc/xianyu-monitor";
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

export function XianyuMonitorPage() {
  const [accounts, setAccounts] = useState<XianyuAccount[]>([]);
  const [aiAccountOptions, setAiAccountOptions] = useState<AiAccountOption[]>([]);
  const [tasks, setTasks] = useState<MonitorTask[]>([]);
  const [selectedId, setSelectedId] = useState<string>("");
  const [results, setResults] = useState<MonitorResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [generating, setGenerating] = useState(false);
  const [form, setForm] = useState(EMPTY_FORM);

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

  const loadResults = useCallback(async (taskId: string) => {
    if (!taskId) {
      setResults([]);
      return;
    }
    const list = await monitorResultList(OWNER_ID, taskId);
    setResults(list);
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
    if (!selectedId) return;
    void loadResults(selectedId).catch((error) =>
      toast.error(error instanceof Error ? error.message : String(error)),
    );
  }, [selectedId, loadResults]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listenMonitorMatch((payload) => {
      toast.success(`监控命中：${payload.title}`, {
        description: payload.reason,
      });
      if (payload.taskId === selectedId) {
        void loadResults(payload.taskId);
      }
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, [selectedId, loadResults]);

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

  async function handleRun(taskId: string) {
    try {
      const summary = await monitorTaskRun(OWNER_ID, taskId);
      toast.success(
        `扫描 ${summary.scanned} 条，新增 ${summary.newItems} 条，推荐 ${summary.recommended} 条`,
      );
      await loadTasks();
      await loadResults(taskId);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  async function handleDelete(taskId: string) {
    try {
      await monitorTaskDelete(OWNER_ID, taskId);
      toast.success("任务已删除");
      if (selectedId === taskId) {
        setSelectedId("");
        resetForm();
      }
      await loadTasks();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  return (
    <PageScaffold
      title="商品监控"
      subtitle="定时多任务并发：AI 生成关键词 → 列表搜索 → AI 决策 → SQLite 存储 → 桌面通知（AI 余额不足时自动切换备用账号）"
    >
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
          <div className="space-y-3 rounded-xl border border-border bg-card p-4">
            <h2 className="text-sm font-medium">{selectedTask ? "编辑任务" : "新建任务"}</h2>
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
                    <p className="text-xs text-muted-foreground">未配置时将按 AI 配置中的账号自动尝试。</p>
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
                  <label className="text-xs text-muted-foreground">搜索关键词（可 AI 生成）</label>
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
              {selectedTask ? (
                <>
                  <AsyncButton
                    variant="outline"
                    onClick={() => void handleRun(selectedTask.id)}
                    disabled={selectedTask.isRunning}
                  >
                    <Play className="mr-1.5 size-4" />
                    立即运行
                  </AsyncButton>
                  <Button
                    variant="outline"
                    onClick={() => void handleDelete(selectedTask.id)}
                  >
                    <Trash2 className="mr-1.5 size-4" />
                    删除
                  </Button>
                </>
              ) : null}
            </div>
            {selectedTask?.lastError ? (
              <p className="text-xs text-destructive">上次错误：{selectedTask.lastError}</p>
            ) : null}
          </div>

          <div className="space-y-3">
            <h2 className="text-sm font-medium">
              监控结果{selectedTask ? ` — ${selectedTask.name}` : ""}
            </h2>
            {results.length === 0 ? (
              <p className="text-sm text-muted-foreground">暂无结果，保存并运行任务后会写入 SQLite。</p>
            ) : (
              <ul className="space-y-3">
                {results.map((item) => (
                  <li key={item.id}>
                    <article
                      className={`rounded-xl border p-3 sm:p-4 ${
                        item.aiRecommended
                          ? "border-primary/40 bg-primary/5"
                          : "border-border bg-card"
                      }`}
                    >
                      <div className="flex items-start justify-between gap-3">
                        <div className="min-w-0 space-y-1">
                          <a
                            href={item.url}
                            target="_blank"
                            rel="noreferrer"
                            className="line-clamp-2 text-sm font-medium hover:underline"
                          >
                            {item.title}
                          </a>
                          <div className="flex flex-wrap gap-x-3 gap-y-1 text-xs text-muted-foreground">
                            <span className="font-semibold text-foreground">
                              {item.priceText || "—"}
                            </span>
                            {item.sellerName ? <span>{item.sellerName}</span> : null}
                            {item.location ? <span>{item.location}</span> : null}
                            <span>{item.crawledAt}</span>
                          </div>
                          <p className="text-xs text-muted-foreground">{item.aiReason}</p>
                        </div>
                        <a
                          href={item.url}
                          target="_blank"
                          rel="noreferrer"
                          className="shrink-0 text-muted-foreground hover:text-foreground"
                        >
                          <ExternalLink className="size-4" />
                        </a>
                      </div>
                    </article>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </section>
      </div>
    </PageScaffold>
  );
}
