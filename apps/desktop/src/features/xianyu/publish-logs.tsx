/**
 * 闲鱼发布日志页（迁移自原前端 `pages/product-publish/PublishLogs.tsx`）。
 *
 * 按原前端核心交互重写：发布记录分页列表 + 账号/状态筛选 + 清空（保留近 10 天）。
 * 数据访问走 Tauri IPC（`@desk/platform/ipc/publish-log`），复用 crates/app PublishLogService。
 */

import { useEffect, useState } from "react";
import {
  Button,
  ConfirmModal,
  Loading,
  PageScaffold,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  toast,
} from "@desk/ui";
import { ExternalLink, RefreshCw, ScrollText, Trash2 } from "@desk/ui/icons";
import { accountList, type XianyuAccount } from "@desk/platform/ipc/account";
import { publishLogClear, publishLogList, type PublishLog } from "@desk/platform/ipc/publish-log";

const OWNER_ID = 1; // 桌面单用户；多用户时由登录态注入
const PAGE_SIZE_OPTIONS = [10, 20, 50, 100];

const STATUS_CONFIG: Record<string, { label: string; cls: string }> = {
  pending: { label: "待处理", cls: "bg-muted text-muted-foreground" },
  publishing: { label: "发布中", cls: "bg-amber-500/15 text-amber-600" },
  success: { label: "成功", cls: "bg-emerald-500/15 text-emerald-600" },
  failed: { label: "失败", cls: "bg-red-500/15 text-red-600" },
};

const ADDRESS_SOURCE_CONFIG: Record<string, { label: string; cls: string }> = {
  material: { label: "素材地址", cls: "bg-primary/10 text-primary" },
  account_pool: { label: "账号随机", cls: "bg-amber-500/15 text-amber-600" },
  global_pool: { label: "全局随机", cls: "bg-muted text-muted-foreground" },
  personal_pool: { label: "个人随机", cls: "bg-primary/10 text-primary" },
};

/**
 * 闲鱼发布日志页。
 *
 * @author agent
 * @created 2026-08-13
 */
export function XianyuPublishLogsPage() {
  const [logs, setLogs] = useState<PublishLog[]>([]);
  const [accounts, setAccounts] = useState<XianyuAccount[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [loading, setLoading] = useState(true);
  const [tableLoading, setTableLoading] = useState(false);
  const [filterAccount, setFilterAccount] = useState("");
  const [filterStatus, setFilterStatus] = useState("");
  const [clearConfirm, setClearConfirm] = useState(false);
  const [clearing, setClearing] = useState(false);

  async function load(nextPage = page, nextPageSize = pageSize, account = filterAccount, status = filterStatus) {
    setTableLoading(true);
    try {
      const [list, count] = await publishLogList({
        page: nextPage,
        page_size: nextPageSize,
        account_id: account || undefined,
        status: status || undefined,
      });
      setLogs(list);
      setTotal(count);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setLoading(false);
      setTableLoading(false);
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
    void publishLogList({ page: 1, page_size: 20 })
      .then(([list, count]) => {
        if (cancelled) return;
        setLogs(list);
        setTotal(count);
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
    };
  }, []);

  async function handleClear() {
    setClearing(true);
    try {
      await publishLogClear(10);
      toast.success("已清空10天前的日志");
      setClearConfirm(false);
      await load(1);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setClearing(false);
    }
  }

  const accountLabel = (accountId: string) => {
    const account = accounts.find((item) => item.account_id === accountId);
    return account?.remark ? `${accountId} (${account.remark})` : accountId;
  };

  return (
    <PageScaffold subtitle="闲鱼发布日志 — 查看所有商品发布记录及结果">
      <div className="space-y-4">
        {/* 工具栏 */}
        <div className="flex items-center justify-between gap-3">
          <div className="flex gap-2">
            <Button
              size="sm"
              variant="outline"
              className="text-destructive"
              onClick={() => setClearConfirm(true)}
              disabled={tableLoading || clearing}
            >
              <Trash2 className="size-4" aria-hidden />
              清空日志
            </Button>
            <Button size="sm" variant="outline" onClick={() => void load()} disabled={tableLoading}>
              <RefreshCw className="size-4" aria-hidden />
              刷新
            </Button>
          </div>
        </div>

        {/* 筛选栏 */}
        <div className="rounded-xl border border-border bg-shell p-4">
          <div className="flex flex-wrap items-end gap-3">
            <label className="block space-y-1">
              <span className="text-[length:var(--text-xs)] text-muted-foreground">筛选账号</span>
              <Select value={filterAccount} onValueChange={setFilterAccount}>
                <SelectTrigger className="w-52" aria-label="筛选账号">
                  <SelectValue placeholder="所有账号" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="">所有账号</SelectItem>
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
            <label className="block space-y-1">
              <span className="text-[length:var(--text-xs)] text-muted-foreground">发布状态</span>
              <Select value={filterStatus} onValueChange={setFilterStatus}>
                <SelectTrigger className="w-36" aria-label="发布状态">
                  <SelectValue placeholder="所有状态" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="">所有状态</SelectItem>
                  {Object.entries(STATUS_CONFIG).map(([key, config]) => (
                    <SelectItem key={key} value={key}>
                      {config.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </label>
            <div className="flex items-end gap-2">
              <Button
                size="sm"
                onClick={() => {
                  setPage(1);
                  void load(1, pageSize, filterAccount, filterStatus);
                }}
                disabled={tableLoading}
              >
                查询
              </Button>
              {filterAccount || filterStatus ? (
                <Button
                  size="sm"
                  variant="outline"
                  className="text-destructive"
                  onClick={() => {
                    setFilterAccount("");
                    setFilterStatus("");
                    setPage(1);
                    void load(1, pageSize, "", "");
                  }}
                  disabled={tableLoading}
                >
                  重置
                </Button>
              ) : null}
            </div>
          </div>
        </div>

        {/* 日志表格 */}
        {loading ? (
          <Loading size="lg" text="加载中..." className="py-16" />
        ) : logs.length === 0 ? (
          <div className="py-16 text-center text-muted-foreground">
            <ScrollText className="mx-auto mb-2 size-10 opacity-40" aria-hidden />
            暂无发布记录
          </div>
        ) : (
          <div className="overflow-hidden rounded-xl border border-border">
            <div className="flex items-center justify-between border-b border-border bg-muted/30 px-4 py-2">
              <span className="text-[length:var(--text-sm)] font-medium">发布记录</span>
              <span className="rounded-full bg-primary/15 px-2 py-0.5 text-[length:var(--text-xs)] text-primary">
                共 {total} 条
              </span>
            </div>
            <div className="max-h-[60vh] overflow-auto">
              <table className="w-full text-[length:var(--text-xs)]">
                <thead className="sticky top-0 bg-muted/80 text-muted-foreground">
                  <tr>
                    <th className="px-3 py-2 text-left font-medium">账号</th>
                    <th className="px-3 py-2 text-left font-medium">商品标题</th>
                    <th className="px-3 py-2 text-left font-medium">价格</th>
                    <th className="px-3 py-2 text-left font-medium">所在地</th>
                    <th className="px-3 py-2 text-left font-medium">状态</th>
                    <th className="px-3 py-2 text-left font-medium">结果 / 错误</th>
                    <th className="px-3 py-2 text-left font-medium">发布时间</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border">
                  {tableLoading ? (
                    <tr>
                      <td colSpan={7} className="py-12 text-center text-muted-foreground">加载中...</td>
                    </tr>
                  ) : (
                    logs.map((log) => {
                      const status = STATUS_CONFIG[log.status] ?? { label: log.status, cls: "bg-muted text-muted-foreground" };
                      const addressSource = log.address_source
                        ? (ADDRESS_SOURCE_CONFIG[log.address_source] ?? { label: log.address_source, cls: "bg-muted text-muted-foreground" })
                        : null;
                      return (
                        <tr key={log.id} className="hover:bg-muted/30">
                          <td className="whitespace-nowrap px-3 py-2 font-medium text-primary">
                            {accountLabel(log.account_id)}
                          </td>
                          <td className="max-w-48 px-3 py-2">
                            <span className="block truncate" title={log.title}>
                              {log.title}
                            </span>
                          </td>
                          <td className="whitespace-nowrap px-3 py-2 font-medium text-amber-600">
                            {log.price ? `¥${log.price}` : "-"}
                          </td>
                          <td className="max-w-52 px-3 py-2">
                            {log.resolved_address_text ? (
                              <div className="space-y-1">
                                <span className="block truncate" title={log.resolved_address_text}>
                                  {log.resolved_address_text}
                                </span>
                                {addressSource ? (
                                  <span className={`rounded-full px-2 py-0.5 text-[length:var(--text-xs)] ${addressSource.cls}`}>
                                    {addressSource.label}
                                  </span>
                                ) : null}
                              </div>
                            ) : (
                              <span className="text-muted-foreground">-</span>
                            )}
                          </td>
                          <td className="px-3 py-2">
                            <span className={`rounded-full px-2 py-0.5 text-[length:var(--text-xs)] ${status.cls}`}>
                              {status.label}
                            </span>
                          </td>
                          <td className="max-w-48 px-3 py-2">
                            {log.item_url ? (
                              <a
                                href={log.item_url}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="flex items-center gap-1 text-primary hover:underline"
                              >
                                <ExternalLink className="size-3" aria-hidden />
                                查看商品
                              </a>
                            ) : log.error_message ? (
                              <span className="block truncate text-red-600" title={log.error_message}>
                                {log.error_message}
                              </span>
                            ) : (
                              <span className="text-muted-foreground">-</span>
                            )}
                          </td>
                          <td className="whitespace-nowrap px-3 py-2 text-muted-foreground">
                            {log.created_at ?? "-"}
                          </td>
                        </tr>
                      );
                    })
                  )}
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
              <Select
                value={String(pageSize)}
                onValueChange={(value) => {
                  setPageSize(Number(value));
                  setPage(1);
                  void load(1, Number(value));
                }}
              >
                <SelectTrigger className="w-24" aria-label="每页条数">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {PAGE_SIZE_OPTIONS.map((size) => (
                    <SelectItem key={size} value={String(size)}>
                      {size} 条
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <span>共 {total} 条</span>
            </div>
            <div className="flex items-center gap-2">
              <Button
                size="sm"
                variant="outline"
                disabled={page <= 1}
                onClick={() => void load(Math.max(1, page - 1))}
              >
                上一页
              </Button>
              <span className="text-[length:var(--text-sm)] text-muted-foreground">
                第 {page} / {Math.max(1, Math.ceil(total / pageSize))} 页
              </span>
              <Button
                size="sm"
                variant="outline"
                disabled={page >= Math.ceil(total / pageSize)}
                onClick={() => void load(Math.min(Math.ceil(total / pageSize), page + 1))}
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
        title="确认清空日志"
        message="此操作将清空10天前的发布日志数据，最近10天的日志将被保留。确定要继续吗？"
        confirmText="确认清空"
        loading={clearing}
        onConfirm={() => void handleClear()}
        onCancel={() => setClearConfirm(false)}
      />
    </PageScaffold>
  );
}
