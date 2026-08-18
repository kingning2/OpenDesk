/**
 * 闲鱼黑名单管理页（迁移自原前端 `pages/blacklist/PersonalBlacklist.tsx`）。
 *
 * 按原前端核心交互重写：个人黑名单列表 + 买家搜索 + 批量新增 + 启用切换 + 删除。
 * 数据访问走 Tauri IPC（`@desk/platform/ipc/blacklist`），复用 crates/app BlacklistService。
 */

import { useEffect, useState } from "react";
import {
  Button,
  ConfirmModal,
  Input,
  Loading,
  PageScaffold,
  Textarea,
  toast,
} from "@desk/ui";
import { Plus, Trash2 } from "@desk/ui/icons";
import {
  blacklistDelete,
  blacklistLevel,
  blacklistPersonalCreate,
  blacklistPersonalList,
  blacklistSetEnabled,
  type PersonalBlacklistItem,
} from "@desk/platform/ipc/blacklist";

const OWNER_ID = 1; // 桌面单用户；多用户时由登录态注入
const PAGE_SIZE = 20;

/**
 * 闲鱼黑名单管理页。
 *
 * @author agent
 * @created 2026-08-13
 */
export function XianyuBlacklistPage() {
  const [items, setItems] = useState<PersonalBlacklistItem[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [loading, setLoading] = useState(false);
  const [search, setSearch] = useState("");
  const [showForm, setShowForm] = useState(false);
  const [buyerIds, setBuyerIds] = useState("");
  const [reason, setReason] = useState("");
  const [deleteTarget, setDeleteTarget] = useState<PersonalBlacklistItem | null>(null);
  const [saving, setSaving] = useState(false);

  async function load(nextPage = page, nextSearch = search) {
    setLoading(true);
    try {
      const [list, count] = await blacklistPersonalList({
        owner_id: OWNER_ID,
        page: nextPage,
        page_size: PAGE_SIZE,
        buyer_id: nextSearch.trim() || undefined,
      });
      setItems(list);
      setTotal(count);
      setPage(nextPage);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    let cancelled = false;
    void blacklistPersonalList({
      owner_id: OWNER_ID,
      page: 1,
      page_size: PAGE_SIZE,
      buyer_id: search.trim() || undefined,
    })
      .then(([list, count]) => {
        if (cancelled) return;
        setItems(list);
        setTotal(count);
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
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE));

  async function handleCreate() {
    if (!buyerIds.trim()) {
      toast.error("买家 ID 不能为空");
      return;
    }
    setSaving(true);
    try {
      await blacklistPersonalCreate({
        owner_id: OWNER_ID,
        buyer_ids: buyerIds,
        reason: reason.trim() || undefined,
      });
      toast.success("已加入黑名单");
      setShowForm(false);
      setBuyerIds("");
      setReason("");
      await load(1);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

  async function handleToggle(item: PersonalBlacklistItem) {
    try {
      await blacklistSetEnabled(OWNER_ID, item.id, !item.is_enabled);
      toast.success(item.is_enabled ? "已停用" : "已启用");
      await load();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  async function handleDelete() {
    if (!deleteTarget) return;
    try {
      await blacklistDelete(OWNER_ID, deleteTarget.id);
      toast.success("已移除黑名单");
      setDeleteTarget(null);
      await load();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  return (
    <PageScaffold subtitle="闲鱼黑名单管理 — 禁止发货买家">
      <div className="space-y-4">
        {/* 工具栏 */}
        <div className="flex items-center justify-between gap-3">
          <div className="flex items-center gap-3">
            <Input
              placeholder="搜索买家 ID"
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter") void load(1);
              }}
              className="w-56"
            />
            <Button variant="outline" size="sm" onClick={() => void load(1)}>
              搜索
            </Button>
          </div>
          <Button onClick={() => setShowForm(true)}>
            <Plus className="size-4" aria-hidden />
            加入黑名单
          </Button>
        </div>

        {/* 列表 */}
        {loading ? (
          <Loading size="lg" text="加载中..." className="py-16" />
        ) : items.length === 0 ? (
          <div className="py-16 text-center text-muted-foreground">暂无黑名单记录</div>
        ) : (
          <div className="overflow-hidden rounded-xl border border-border">
            <table className="w-full text-[length:var(--text-sm)]">
              <thead className="bg-muted/50 text-muted-foreground">
                <tr>
                  <th className="px-4 py-2.5 text-left font-medium">买家 ID</th>
                  <th className="px-4 py-2.5 text-left font-medium">级别</th>
                  <th className="px-4 py-2.5 text-left font-medium">原因</th>
                  <th className="px-4 py-2.5 text-left font-medium">状态</th>
                  <th className="px-4 py-2.5 text-right font-medium">操作</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border">
                {items.map((item) => (
                  <tr key={item.id} className="hover:bg-muted/30">
                    <td className="px-4 py-2.5 font-mono">{item.buyer_id}</td>
                    <td className="px-4 py-2.5">
                      <span className="rounded-full bg-muted px-2 py-0.5 text-[length:var(--text-xs)]">
                        {blacklistLevel(item)}
                      </span>
                    </td>
                    <td className="px-4 py-2.5 text-muted-foreground">{item.reason || "—"}</td>
                    <td className="px-4 py-2.5">
                      <span
                        className={
                          item.is_enabled
                            ? "rounded-full bg-red-500/15 px-2 py-0.5 text-[length:var(--text-xs)] text-red-500"
                            : "rounded-full bg-muted px-2 py-0.5 text-[length:var(--text-xs)] text-muted-foreground"
                        }
                      >
                        {item.is_enabled ? "生效中" : "已停用"}
                      </span>
                    </td>
                    <td className="px-4 py-2.5 text-right">
                      <div className="flex items-center justify-end gap-1">
                        <Button size="sm" variant="outline" onClick={() => void handleToggle(item)}>
                          {item.is_enabled ? "停用" : "启用"}
                        </Button>
                        <Button
                          size="sm"
                          variant="ghost"
                          className="text-destructive"
                          onClick={() => setDeleteTarget(item)}
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
              onClick={() => void load(Math.max(1, page - 1))}
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
              onClick={() => void load(Math.min(totalPages, page + 1))}
            >
              下一页
            </Button>
          </div>
        ) : null}
      </div>

      {/* 新增黑名单弹窗 */}
      <ConfirmModal
        isOpen={showForm}
        title="加入黑名单"
        message={
          <div className="space-y-3 text-left">
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">
                买家 ID（支持多行批量）
              </span>
              <Textarea
                value={buyerIds}
                onChange={(event) => setBuyerIds(event.target.value)}
                placeholder={"buyer-001\nbuyer-002"}
                rows={3}
              />
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">原因</span>
              <Input
                value={reason}
                onChange={(event) => setReason(event.target.value)}
                placeholder="如：恶意退款买家"
              />
            </label>
          </div>
        }
        confirmText={saving ? "添加中…" : "添加"}
        loading={saving}
        onConfirm={() => void handleCreate()}
        onCancel={() => {
          setShowForm(false);
          setBuyerIds("");
          setReason("");
        }}
      />

      {/* 删除确认 */}
      <ConfirmModal
        isOpen={deleteTarget !== null}
        type="danger"
        title="移除黑名单"
        message={`确认将买家「${deleteTarget?.buyer_id ?? ""}」移出黑名单？`}
        confirmText="移除"
        onConfirm={() => void handleDelete()}
        onCancel={() => setDeleteTarget(null)}
      />
    </PageScaffold>
  );
}
