/**
 * 闲鱼批量发布页（迁移自原前端 `pages/product-publish/BatchPublish.tsx`）。
 *
 * 按原前端核心交互重写：多账号 × 多素材选择 → 提交批量任务 → 轮询进度面板
 * （总数/成功/失败/进行中 + 进度条 + 账号同步状态）。
 * 数据访问走 Tauri IPC（`@desk/platform/ipc/publish-batch`），复用 crates/app BatchService。
 *
 * 说明：任务在壳层后台逐条执行（内存网关模拟发布），前端每 3 秒轮询进度。
 */

import { useEffect, useRef, useState } from "react";
import {
  Button,
  Loading,
  PageScaffold,
  toast,
} from "@desk/ui";
import { CheckCircle, Clock, Layers, Play, XCircle } from "@desk/ui/icons";
import { accountList, type XianyuAccount } from "@desk/platform/ipc/account";
import {
  publishBatchStatus,
  publishBatchSubmit,
  type BatchAccountStatus,
  type BatchTask,
} from "@desk/platform/ipc/publish-batch";
import { publishMaterialList, type PublishMaterial } from "@desk/platform/ipc/publish-material";

const OWNER_ID = 1; // 桌面单用户；多用户时由登录态注入
const POLL_INTERVAL_MS = 3000;

const SYNC_STATUS_LABELS: Record<string, string> = {
  success: "已成功",
  failed: "失败",
  running: "获取中",
  skipped: "未触发",
  unknown: "状态未知",
  pending: "待执行",
};

/** 进度统计卡。 */
function StatCard({
  label,
  value,
  className,
  icon,
}: {
  label: string;
  value: number;
  className: string;
  icon: React.ReactNode;
}) {
  return (
    <div className="flex items-center gap-3 rounded-lg border border-border bg-background p-3">
      <div className={`flex size-9 items-center justify-center rounded-lg ${className}`}>{icon}</div>
      <div>
        <div className="text-lg font-semibold">{value}</div>
        <div className="text-[length:var(--text-xs)] text-muted-foreground">{label}</div>
      </div>
    </div>
  );
}

/**
 * 闲鱼批量发布页。
 *
 * @author agent
 * @created 2026-08-13
 */
export function XianyuBatchPublishPage() {
  const [accounts, setAccounts] = useState<XianyuAccount[]>([]);
  const [materials, setMaterials] = useState<PublishMaterial[]>([]);
  const [selectedAccounts, setSelectedAccounts] = useState<Set<string>>(new Set());
  const [selectedMaterials, setSelectedMaterials] = useState<Set<number>>(new Set());
  const [loading, setLoading] = useState(true);
  const [materialSearch, setMaterialSearch] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [progress, setProgress] = useState<BatchTask | null>(null);
  const pollingRef = useRef<ReturnType<typeof setInterval> | null>(null);

  function stopPolling() {
    if (pollingRef.current) {
      clearInterval(pollingRef.current);
      pollingRef.current = null;
    }
  }

  function startPolling(batchId: string) {
    stopPolling();
    pollingRef.current = setInterval(() => {
      void publishBatchStatus(batchId)
        .then((task) => {
          if (!task) {
            stopPolling();
            setProgress(null);
            toast.warning("批量任务状态已失效，请重新提交任务");
            return;
          }
          setProgress(task);
          if (task.finished) {
            stopPolling();
            toast[task.failed === 0 ? "success" : "warning"](
              `批量发布完成！成功 ${task.success} 条，失败 ${task.failed} 条`,
            );
          }
        })
        .catch(() => {
          // 轮询失败静默，下次再试
        });
    }, POLL_INTERVAL_MS);
  }

  useEffect(() => {
    let cancelled = false;
    void Promise.all([
      accountList(OWNER_ID),
      publishMaterialList({ page: 1, page_size: 100 }),
    ])
      .then(([accountListData, [materialList]]) => {
        if (cancelled) return;
        setAccounts(accountListData);
        setMaterials(materialList);
      })
      .catch((error) => {
        if (!cancelled) {
          toast.error(error instanceof Error ? error.message : String(error));
        }
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
      stopPolling();
    };
  }, []);

  const filteredMaterials = materialSearch.trim()
    ? materials.filter((m) => m.title.toLowerCase().includes(materialSearch.trim().toLowerCase()))
    : materials;

  const total = selectedAccounts.size * selectedMaterials.size;
  const isDisabled = submitting || total === 0 || (progress !== null && !progress.finished);

  function toggleAccount(accountId: string) {
    setSelectedAccounts((prev) => {
      const next = new Set(prev);
      if (next.has(accountId)) {
        next.delete(accountId);
      } else {
        next.add(accountId);
      }
      return next;
    });
  }

  function toggleMaterial(materialId: number) {
    setSelectedMaterials((prev) => {
      const next = new Set(prev);
      if (next.has(materialId)) {
        next.delete(materialId);
      } else {
        next.add(materialId);
      }
      return next;
    });
  }

  function toggleAllAccounts() {
    setSelectedAccounts(
      selectedAccounts.size === accounts.length && accounts.length > 0
        ? new Set()
        : new Set(accounts.map((account) => account.account_id)),
    );
  }

  function toggleAllMaterials() {
    const ids = filteredMaterials.map((m) => m.id);
    const allSelected = ids.length > 0 && ids.every((id) => selectedMaterials.has(id));
    setSelectedMaterials((prev) => {
      const next = new Set(prev);
      if (allSelected) {
        ids.forEach((id) => next.delete(id));
      } else {
        ids.forEach((id) => next.add(id));
      }
      return next;
    });
  }

  async function handleSubmit() {
    if (selectedAccounts.size === 0) {
      toast.warning("请至少选择一个账号");
      return;
    }
    if (selectedMaterials.size === 0) {
      toast.warning("请至少选择一条素材");
      return;
    }
    setSubmitting(true);
    try {
      const task = await publishBatchSubmit(
        Array.from(selectedAccounts),
        Array.from(selectedMaterials),
      );
      toast.success("批量发布任务已提交");
      setProgress(task);
      startPolling(task.batch_id);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSubmitting(false);
    }
  }

  const progressPercent =
    progress && progress.total > 0
      ? Math.round(((progress.success + progress.failed) / progress.total) * 100)
      : 0;

  const accountLabel = (accountId: string) => {
    const account = accounts.find((item) => item.account_id === accountId);
    return account?.remark ? `${accountId} (${account.remark})` : accountId;
  };

  return (
    <PageScaffold subtitle="闲鱼批量发布 — 多账号多素材并发发布，提升发布效率">
      <div className="space-y-4">
        {/* 标题栏 */}
        <div className="flex items-center justify-between gap-3">
          <div>
            <h1 className="font-semibold">批量发布</h1>
            <p className="text-[length:var(--text-xs)] text-muted-foreground">
              批量发布时会忽略素材库中填写的宝贝所在地，统一从随机地址库自动分配地址。
            </p>
          </div>
          <div className="rounded-lg bg-muted px-3 py-1.5 text-[length:var(--text-sm)] text-muted-foreground">
            {selectedAccounts.size} 账号 {selectedMaterials.size} 素材 ={" "}
            <span className="font-semibold text-primary">{total} 次发布</span>
          </div>
        </div>

        {loading ? (
          <Loading size="lg" text="加载中..." className="py-16" />
        ) : (
          <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
            {/* 账号选择 */}
            <div className="rounded-xl border border-border bg-shell p-4">
              <div className="mb-2 flex items-center justify-between">
                <h2 className="font-medium">选择账号</h2>
                <button
                  type="button"
                  onClick={toggleAllAccounts}
                  className="text-[length:var(--text-sm)] text-primary hover:underline"
                >
                  {selectedAccounts.size === accounts.length && accounts.length > 0 ? "取消全选" : "全选"}
                </button>
              </div>
              {accounts.length === 0 ? (
                <p className="py-8 text-center text-muted-foreground">暂无账号，请先添加账号</p>
              ) : (
                <div className="max-h-72 space-y-1 overflow-y-auto">
                  {accounts.map((account) => {
                    const checked = selectedAccounts.has(account.account_id);
                    return (
                      <label
                        key={account.account_id}
                        className={`flex cursor-pointer items-center gap-3 rounded-lg p-2.5 transition-colors ${
                          checked ? "bg-primary/10" : "hover:bg-muted/40"
                        }`}
                      >
                        <input
                          type="checkbox"
                          checked={checked}
                          onChange={() => toggleAccount(account.account_id)}
                          className="size-4 accent-primary"
                        />
                        <div className="min-w-0 flex-1">
                          <p className="truncate text-[length:var(--text-sm)] font-medium">
                            {account.remark || account.account_id}
                          </p>
                          {account.remark ? (
                            <p className="truncate text-[length:var(--text-xs)] text-muted-foreground">
                              {account.account_id}
                            </p>
                          ) : null}
                        </div>
                        <span
                          className={`rounded-full px-2 py-0.5 text-[length:var(--text-xs)] ${
                            account.status === "active"
                              ? "bg-emerald-500/15 text-emerald-600"
                              : "bg-muted text-muted-foreground"
                          }`}
                        >
                          {account.status === "active" ? "已启动" : "未启动"}
                        </span>
                      </label>
                    );
                  })}
                </div>
              )}
            </div>

            {/* 素材选择 */}
            <div className="rounded-xl border border-border bg-shell p-4">
              <div className="mb-2 flex items-center justify-between gap-2">
                <h2 className="font-medium">选择素材</h2>
                <button
                  type="button"
                  onClick={toggleAllMaterials}
                  className="text-[length:var(--text-sm)] text-primary hover:underline"
                >
                  {selectedMaterials.size === filteredMaterials.length && filteredMaterials.length > 0
                    ? "取消全选"
                    : "全选"}
                </button>
              </div>
              <input
                value={materialSearch}
                onChange={(event) => setMaterialSearch(event.target.value)}
                placeholder="搜索素材标题..."
                className="mb-2 w-full rounded-md border border-border bg-background px-3 py-2 text-[length:var(--text-sm)] outline-none focus-visible:ring-2 focus-visible:ring-primary/40"
              />
              {filteredMaterials.length === 0 ? (
                <p className="py-8 text-center text-muted-foreground">
                  {materials.length === 0 ? "素材库为空，请先在「素材库」页面添加素材" : "没有匹配的素材"}
                </p>
              ) : (
                <div className="max-h-72 space-y-1 overflow-y-auto">
                  {filteredMaterials.map((material) => {
                    const checked = selectedMaterials.has(material.id);
                    return (
                      <label
                        key={material.id}
                        className={`flex cursor-pointer items-center gap-3 rounded-lg p-2.5 transition-colors ${
                          checked ? "bg-primary/10" : "hover:bg-muted/40"
                        }`}
                      >
                        <input
                          type="checkbox"
                          checked={checked}
                          onChange={() => toggleMaterial(material.id)}
                          className="size-4 accent-primary"
                        />
                        <div className="min-w-0 flex-1">
                          <p className="truncate text-[length:var(--text-sm)] font-medium">
                            {material.title}
                          </p>
                          <div className="mt-1 flex flex-wrap items-center gap-1.5">
                            <span className="text-[length:var(--text-xs)] text-amber-600">
                              {material.price}
                            </span>
                            <span className="rounded-full bg-muted px-2 py-0.5 text-[length:var(--text-xs)] text-muted-foreground">
                              {material.condition}
                            </span>
                          </div>
                        </div>
                      </label>
                    );
                  })}
                </div>
              )}
            </div>
          </div>
        )}

        {/* 提交按钮 */}
        <div className="flex justify-center">
          <Button className="min-w-48" disabled={isDisabled} onClick={() => void handleSubmit()}>
            <Play className="size-4" aria-hidden />
            {submitting ? "提交中..." : `开始批量发布（${total} 次）`}
          </Button>
        </div>

        {/* 进度面板 */}
        {progress ? (
          <div className="rounded-xl border border-border bg-shell p-4">
            <div className="mb-3 flex items-center justify-between">
              <h2 className="flex items-center gap-2 font-medium">
                <Layers className="size-4" aria-hidden />
                发布进度
              </h2>
              {progress.finished ? (
                <span className="rounded-full bg-emerald-500/15 px-2 py-0.5 text-[length:var(--text-xs)] text-emerald-600">
                  已完成
                </span>
              ) : (
                <span className="text-[length:var(--text-xs)] text-muted-foreground">执行中...</span>
              )}
            </div>
            <div className="mb-4 grid grid-cols-2 gap-3 sm:grid-cols-4">
              <StatCard label="总数" value={progress.total} className="bg-primary/10 text-primary" icon={<Layers className="size-4" aria-hidden />} />
              <StatCard label="成功" value={progress.success} className="bg-emerald-500/15 text-emerald-600" icon={<CheckCircle className="size-4" aria-hidden />} />
              <StatCard label="失败" value={progress.failed} className="bg-red-500/15 text-red-600" icon={<XCircle className="size-4" aria-hidden />} />
              <StatCard label="进行中" value={progress.publishing + progress.pending} className="bg-blue-500/15 text-blue-600" icon={<Clock className="size-4" aria-hidden />} />
            </div>
            {progress.total > 0 ? (
              <>
                <div className="mb-1 h-2 w-full rounded-full bg-muted">
                  <div
                    className="h-2 rounded-full bg-primary transition-all duration-500"
                    style={{ width: `${progressPercent}%` }}
                  />
                </div>
                <div className="flex justify-between text-[length:var(--text-xs)] text-muted-foreground">
                  <span>进度 {progressPercent}%</span>
                  <span>批次 ID：{progress.batch_id.slice(0, 8)}...</span>
                </div>
              </>
            ) : null}
            {!progress.finished ? (
              <p className="mt-2 text-[length:var(--text-xs)] text-muted-foreground">
                每 3 秒自动刷新进度
              </p>
            ) : null}
            {progress.account_statuses.length > 0 ? (
              <div className="mt-4 space-y-2 border-t border-border pt-4">
                <div className="flex items-center justify-between gap-2">
                  <h3 className="text-[length:var(--text-sm)] font-medium">账号自动获取商品状态</h3>
                  <span className="text-[length:var(--text-xs)] text-muted-foreground">
                    按账号展示发布后商品同步结果
                  </span>
                </div>
                <div className="max-h-72 space-y-2 overflow-y-auto pr-1">
                  {progress.account_statuses.map((status: BatchAccountStatus) => (
                    <div key={status.account_id} className="rounded-xl border border-border p-3">
                      <div className="flex flex-col justify-between gap-2 sm:flex-row sm:items-center">
                        <div className="min-w-0">
                          <div className="truncate text-[length:var(--text-sm)] font-medium">
                            {accountLabel(status.account_id)}
                          </div>
                          <div className="truncate text-[length:var(--text-xs)] text-muted-foreground">
                            账号ID：{status.account_id}
                          </div>
                        </div>
                        <span
                          className={`w-fit rounded-full px-2 py-0.5 text-[length:var(--text-xs)] ${
                            status.sync_status === "success"
                              ? "bg-emerald-500/15 text-emerald-600"
                              : status.sync_status === "failed"
                                ? "bg-red-500/15 text-red-600"
                                : "bg-muted text-muted-foreground"
                          }`}
                        >
                          {SYNC_STATUS_LABELS[status.sync_status] ?? status.sync_status}
                        </span>
                      </div>
                      <div className="mt-3 grid grid-cols-2 gap-2 text-[length:var(--text-xs)] sm:grid-cols-4">
                        <div className="rounded-lg bg-muted/40 px-2.5 py-2">
                          <div className="text-muted-foreground">发布总数</div>
                          <div className="mt-1 font-semibold">{status.total}</div>
                        </div>
                        <div className="rounded-lg bg-muted/40 px-2.5 py-2">
                          <div className="text-muted-foreground">发布成功</div>
                          <div className="mt-1 font-semibold text-emerald-600">{status.success}</div>
                        </div>
                        <div className="rounded-lg bg-muted/40 px-2.5 py-2">
                          <div className="text-muted-foreground">发布失败</div>
                          <div className="mt-1 font-semibold text-amber-600">{status.failed}</div>
                        </div>
                        <div className="rounded-lg bg-muted/40 px-2.5 py-2">
                          <div className="text-muted-foreground">待处理</div>
                          <div className="mt-1 font-semibold text-blue-600">
                            {status.publishing + status.pending}
                          </div>
                        </div>
                      </div>
                      <div className="mt-3 break-all text-[length:var(--text-xs)] text-muted-foreground">
                        {status.sync_message}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            ) : null}
          </div>
        ) : null}
      </div>
    </PageScaffold>
  );
}
