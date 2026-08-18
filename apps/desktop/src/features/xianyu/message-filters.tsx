/**
 * 闲鱼消息过滤管理页（迁移自原前端 `pages/messageFilters/MessageFilters.tsx`）。
 *
 * 按原前端核心交互重写：按账号查看过滤规则 + 新增 / 编辑 / 删除 / 启用切换。
 * 数据访问走 Tauri IPC（`@desk/platform/ipc/filter`），复用 crates/app FilterService。
 *
 * 与后端的对齐说明：Rust `FilterService` 按账号查询（owner_id + account_id），
 * 因此页面按账号隔离（原前端「全部账号」视图由多账号筛选承担）。
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
import { Pencil, Plus, Trash2 } from "@desk/ui/icons";
import { accountList, type XianyuAccount } from "@desk/platform/ipc/account";
import {
  FILTER_TYPE_LABELS,
  FILTER_TYPE_OPTIONS,
  filterCreate,
  filterDelete,
  filterList,
  filterSetEnabled,
  filterUpdate,
  type FilterRule,
  type FilterType,
} from "@desk/platform/ipc/filter";

const OWNER_ID = 1; // 桌面单用户；多用户时由登录态注入
const PAGE_SIZE = 20;

/** 表单值（新增 / 编辑共用）。 */
interface FilterFormState {
  keyword: string;
  filter_type: FilterType;
}

const EMPTY_FORM: FilterFormState = { keyword: "", filter_type: "skip_reply" };

/**
 * 闲鱼消息过滤管理页。
 *
 * @author agent
 * @created 2026-08-13
 */
export function XianyuMessageFiltersPage() {
  const [accounts, setAccounts] = useState<XianyuAccount[]>([]);
  const [accountId, setAccountId] = useState("");
  const [filters, setFilters] = useState<FilterRule[]>([]);
  const [loading, setLoading] = useState(true);
  const [formOpen, setFormOpen] = useState(false);
  const [editing, setEditing] = useState<FilterRule | null>(null);
  const [form, setForm] = useState<FilterFormState>(EMPTY_FORM);
  const [saving, setSaving] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<FilterRule | null>(null);
  const [page, setPage] = useState(1);

  // 加载账号列表（默认选中第一个账号）。
  useEffect(() => {
    let cancelled = false;
    void accountList(OWNER_ID)
      .then((list) => {
        if (cancelled) return;
        setAccounts(list);
        if (list.length > 0) {
          setAccountId((current) => current || list[0].account_id);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          toast.error(error instanceof Error ? error.message : String(error));
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  // 按账号加载过滤规则。
  useEffect(() => {
    if (!accountId) return;
    let cancelled = false;
    void filterList(OWNER_ID, accountId)
      .then((list) => {
        if (cancelled) return;
        setFilters(list);
        setPage(1);
      })
      .catch((error) => {
        if (!cancelled) {
          toast.error(error instanceof Error ? error.message : String(error));
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
  }, [accountId]);

  const totalPages = Math.max(1, Math.ceil(filters.length / PAGE_SIZE));
  const paginated = filters.slice((page - 1) * PAGE_SIZE, page * PAGE_SIZE);

  function openCreate() {
    if (!accountId) {
      toast.warning("请先选择账号");
      return;
    }
    setEditing(null);
    setForm(EMPTY_FORM);
    setFormOpen(true);
  }

  function openEdit(rule: FilterRule) {
    setEditing(rule);
    setForm({ keyword: rule.keyword, filter_type: rule.filter_type });
    setFormOpen(true);
  }

  async function handleSubmit() {
    const keyword = form.keyword.trim();
    if (!keyword) {
      toast.error("过滤关键词不能为空");
      return;
    }
    setSaving(true);
    try {
      if (editing) {
        await filterUpdate(OWNER_ID, {
          id: editing.id,
          account_id: editing.account_id,
          filter_type: form.filter_type,
          keyword,
        });
        toast.success("规则已更新");
      } else {
        await filterCreate(OWNER_ID, accountId, {
          filter_type: form.filter_type,
          keyword,
        });
        toast.success("规则已创建");
      }
      setFormOpen(false);
      await reload();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

  async function reload() {
    if (!accountId) return;
    try {
      const list = await filterList(OWNER_ID, accountId);
      setFilters(list);
      setPage(1);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  async function handleToggle(rule: FilterRule) {
    try {
      await filterSetEnabled(OWNER_ID, rule.id, !rule.enabled);
      toast.success(rule.enabled ? "已停用" : "已启用");
      await reload();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  async function handleDelete() {
    if (!deleteTarget) return;
    try {
      await filterDelete(OWNER_ID, deleteTarget.id);
      toast.success("规则已删除");
      setDeleteTarget(null);
      await reload();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  return (
    <PageScaffold subtitle="闲鱼消息过滤 — 命中关键词时跳过自动回复 / 消息通知">
      <div className="space-y-4">
        {/* 工具栏：账号选择 + 新增 */}
        <div className="flex items-center justify-between gap-3">
          <div className="w-64">
            <Select
              value={accountId}
              onValueChange={(value) => {
                setLoading(true);
                setAccountId(value);
              }}
            >
              <SelectTrigger aria-label="选择账号">
                <SelectValue placeholder="选择账号" />
              </SelectTrigger>
              <SelectContent>
                {accounts.map((account) => (
                  <SelectItem key={account.account_id} value={account.account_id}>
                    {account.remark
                      ? `${account.account_id} (${account.remark})`
                      : account.account_id}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <Button onClick={openCreate} disabled={!accountId}>
            <Plus className="size-4" aria-hidden />
            新建规则
          </Button>
        </div>

        {/* 规则列表 */}
        {loading ? (
          <Loading size="lg" text="加载中..." className="py-16" />
        ) : !accountId ? (
          <div className="py-16 text-center text-muted-foreground">暂无账号，请先创建账号</div>
        ) : filters.length === 0 ? (
          <div className="py-16 text-center text-muted-foreground">暂无过滤规则</div>
        ) : (
          <div className="overflow-hidden rounded-xl border border-border">
            <table className="w-full text-[length:var(--text-sm)]">
              <thead className="bg-muted/50 text-muted-foreground">
                <tr>
                  <th className="px-4 py-2.5 text-left font-medium">关键词</th>
                  <th className="px-4 py-2.5 text-left font-medium">过滤类型</th>
                  <th className="px-4 py-2.5 text-left font-medium">状态</th>
                  <th className="px-4 py-2.5 text-right font-medium">操作</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border">
                {paginated.map((rule) => (
                  <tr key={rule.id} className="hover:bg-muted/30">
                    <td className="px-4 py-2.5 font-mono">{rule.keyword}</td>
                    <td className="px-4 py-2.5">
                      <span className="rounded-full bg-muted px-2 py-0.5 text-[length:var(--text-xs)]">
                        {FILTER_TYPE_LABELS[rule.filter_type]}
                      </span>
                    </td>
                    <td className="px-4 py-2.5">
                      <span
                        className={
                          rule.enabled
                            ? "rounded-full bg-emerald-500/15 px-2 py-0.5 text-[length:var(--text-xs)] text-emerald-600"
                            : "rounded-full bg-muted px-2 py-0.5 text-[length:var(--text-xs)] text-muted-foreground"
                        }
                      >
                        {rule.enabled ? "启用" : "停用"}
                      </span>
                    </td>
                    <td className="px-4 py-2.5 text-right">
                      <div className="flex items-center justify-end gap-1">
                        <Button size="sm" variant="outline" onClick={() => void handleToggle(rule)}>
                          {rule.enabled ? "停用" : "启用"}
                        </Button>
                        <Button size="sm" variant="ghost" onClick={() => openEdit(rule)}>
                          <Pencil className="size-3.5" aria-hidden />
                        </Button>
                        <Button
                          size="sm"
                          variant="ghost"
                          className="text-destructive"
                          onClick={() => setDeleteTarget(rule)}
                        >
                          <Trash2 className="size-3.5" aria-hidden />
                        </Button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        {/* 分页 */}
        {totalPages > 1 ? (
          <div className="flex items-center justify-end gap-2">
            <Button
              size="sm"
              variant="outline"
              disabled={page <= 1}
              onClick={() => setPage((current) => Math.max(1, current - 1))}
            >
              上一页
            </Button>
            <span className="text-[length:var(--text-sm)] text-muted-foreground">
              {page} / {totalPages}
            </span>
            <Button
              size="sm"
              variant="outline"
              disabled={page >= totalPages}
              onClick={() => setPage((current) => Math.min(totalPages, current + 1))}
            >
              下一页
            </Button>
          </div>
        ) : null}
      </div>

      {/* 新增 / 编辑弹窗 */}
      <ConfirmModal
        isOpen={formOpen}
        title={editing ? "编辑过滤规则" : "新建过滤规则"}
        message={
          <div className="space-y-3 text-left">
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">
                过滤关键词
              </span>
              <Input
                value={form.keyword}
                onChange={(event) => setForm((current) => ({ ...current, keyword: event.target.value }))}
                placeholder="如：勿扰、广告"
              />
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">过滤类型</span>
              <Select
                value={form.filter_type}
                onValueChange={(value) =>
                  setForm((current) => ({ ...current, filter_type: value as FilterType }))
                }
              >
                <SelectTrigger aria-label="过滤类型">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {FILTER_TYPE_OPTIONS.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </label>
          </div>
        }
        confirmText={saving ? "保存中…" : "保存"}
        loading={saving}
        onConfirm={() => void handleSubmit()}
        onCancel={() => {
          setFormOpen(false);
          setEditing(null);
        }}
      />

      {/* 删除确认 */}
      <ConfirmModal
        isOpen={deleteTarget !== null}
        type="danger"
        title="删除过滤规则"
        message={`确认删除关键词「${deleteTarget?.keyword ?? ""}」的过滤规则？删除后无法恢复。`}
        confirmText="删除"
        onConfirm={() => void handleDelete()}
        onCancel={() => setDeleteTarget(null)}
      />
    </PageScaffold>
  );
}
