/**
 * 闲鱼商品监控 — 任务 / 运行记录管理；点击某次运行进入详情页（agent 转录式）。
 *
 * 本文件只做编排：数据加载 + 组合 StatsSection / TaskList / RunRecordsSection / TaskForm。
 */
import { OWNER_ID } from "@desk/platform/constants";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { AsyncButton, Button, PageScaffold, toast } from "@desk/ui";
import { Play, Trash2 } from "@desk/ui/icons";
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
import { useWorkspaceNav } from "../../../../app/use-workspace-tabs";
import { BUILT_IN_PROVIDERS } from "@feature/agent/builtin-providers";
import { getErrorMessage } from "@desk/utils";
import { StatsSection } from "./stats";
import { TaskList } from "./task-list";
import { RunRecordsSection } from "./run-records";
import { EMPTY_FORM, TaskForm, type AiAccountOption, type MonitorForm } from "./task-form";

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

/** 闲鱼商品监控页。 */
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
  const [form, setForm] = useState<MonitorForm>(EMPTY_FORM);
  const [stats, setStats] = useState<MonitorStats | null>(null);
  const pendingNavTaskId = useRef<string | null>(null);

  const accountOptions = useMemo(() => toAccountOptions(accounts), [accounts]);
  const aiAccountLabelMap = useMemo(
    () => new Map(aiAccountOptions.map((option) => [option.id, option.label])),
    [aiAccountOptions],
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
        if (!cancelled) toast.error(getErrorMessage(error));
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
        if (!cancelled) toast.error(getErrorMessage(error));
      });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    setLoading(true);
    void loadTasks()
      .catch((error) => toast.error(getErrorMessage(error)))
      .finally(() => setLoading(false));
  }, [loadTasks]);

  useEffect(() => {
    void loadStats().catch((error) => toast.error(getErrorMessage(error)));
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
      .catch((error) => toast.error(getErrorMessage(error)));
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
        aiAccountOrder: form.aiAccountOrder,
        intervalMinutes: Number(form.intervalMinutes) || 5,
        enabled: form.enabled,
        aiCriteria: form.aiCriteria.trim(),
      });
      toast.success("监控任务已保存");
      await loadTasks();
      setSelectedId(saved.id);
      setEditing(false);
    } catch (error) {
      toast.error(getErrorMessage(error));
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
        aiAccountOrder: form.aiAccountOrder,
      });
      setForm((current) => ({ ...current, keywords: keywords.join("\n") }));
      toast.success(`已生成 ${keywords.length} 个关键词`);
    } catch (error) {
      toast.error(getErrorMessage(error));
    } finally {
      setGenerating(false);
    }
  }

  function handleRun(taskId: string) {
    pendingNavTaskId.current = taskId;
    setRunningTaskId(taskId);
    void monitorTaskRun(OWNER_ID, taskId).catch((error) => {
      toast.error(getErrorMessage(error));
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
      toast.error(getErrorMessage(error));
    }
  }

  return (
    <PageScaffold
      title="商品监控"
      subtitle="定时多任务并发：AI 生成关键词 → 列表搜索 → AI 决策 → 结果落库；点击某次运行进入详情页查看完整过程与商品。"
    >
      {stats ? <StatsSection stats={stats} /> : null}

      <div className="grid gap-6 lg:grid-cols-[280px_minmax(0,1fr)]">
        <TaskList
          tasks={tasks}
          selectedId={selectedId}
          loading={loading}
          onSelect={(task) => {
            setSelectedId(task.id);
            resetForm(task);
            setEditing(false);
          }}
          onNew={() => {
            setSelectedId("");
            resetForm();
            setEditing(true);
          }}
        />

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
            <RunRecordsSection
              runs={runs}
              runningTaskId={runningTaskId}
              onOpen={(runId) => selectTab(`${managePath("monitor")}/runs/${runId}`)}
            />
          ) : null}

          {editing || !selectedTask ? (
            <TaskForm
              form={form}
              editingExisting={!!selectedTask}
              accountOptions={accountOptions}
              aiAccountOptions={aiAccountOptions}
              aiAccountLabelMap={aiAccountLabelMap}
              saving={saving}
              generating={generating}
              setForm={setForm}
              onSave={() => void handleSave()}
              onGenerateKeywords={() => void handleGenerateKeywords()}
              onCancel={() => setEditing(false)}
            />
          ) : null}
        </section>
      </div>
    </PageScaffold>
  );
}
