/**
 * 闲鱼商品管理页（迁移自原前端 `pages/items/Items.tsx`）。
 *
 * 按原前端核心交互重写：商品列表 + 关键词/账号筛选 + 擦亮/多规格标记 + AI 提示词编辑。
 * 数据访问走 Tauri IPC（`@desk/platform/ipc/item`），复用 crates/app ItemService。
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
  Textarea,
  toast,
} from "@desk/ui";
import { Pencil, Search } from "@desk/ui/icons";
import { itemList, itemUpdate, type Item } from "@desk/platform/ipc/item";
import { accountList, type XianyuAccount } from "@desk/platform/ipc/account";
import { formatAmount } from "@desk/utils";

const OWNER_ID = 1; // 桌面单用户；多用户时由登录态注入
const PAGE_SIZE = 20;

/**
 * 闲鱼商品管理页。
 *
 * @author agent
 * @created 2026-08-13
 */
export function XianyuItemsPage() {
  const [items, setItems] = useState<Item[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [loading, setLoading] = useState(false);
  const [keyword, setKeyword] = useState("");
  const [accounts, setAccounts] = useState<XianyuAccount[]>([]);
  const [accountId, setAccountId] = useState("all");
  const [editTarget, setEditTarget] = useState<Item | null>(null);
  const [aiPrompt, setAiPrompt] = useState("");
  const [saving, setSaving] = useState(false);

  // 加载账号列表。
  useEffect(() => {
    let cancelled = false;
    void accountList(OWNER_ID)
      .then((list) => {
        if (!cancelled) {
          setAccounts(list);
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

  async function load(nextPage = page) {
    setLoading(true);
    try {
      const [list, count] = await itemList({
        owner_id: OWNER_ID,
        page: nextPage,
        page_size: PAGE_SIZE,
        keyword: keyword.trim() || undefined,
        account_id: accountId === "all" ? undefined : accountId,
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
    void itemList({
      owner_id: OWNER_ID,
      page: 1,
      page_size: PAGE_SIZE,
      keyword: keyword.trim() || undefined,
      account_id: accountId === "all" ? undefined : accountId,
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
  }, [accountId]);

  const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE));

  async function handleSaveAiPrompt() {
    if (!editTarget) return;
    setSaving(true);
    try {
      await itemUpdate(OWNER_ID, editTarget.item_id, aiPrompt);
      toast.success("AI 提示词已保存");
      setEditTarget(null);
      await load();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

  return (
    <PageScaffold subtitle="闲鱼商品管理 — 列表 / 筛选 / AI 提示词">
      <div className="space-y-4">
        {/* 工具栏 */}
        <div className="flex items-center justify-between gap-3">
          <div className="flex items-center gap-3">
            <Input
              placeholder="搜索商品 ID / 标题"
              value={keyword}
              onChange={(event) => setKeyword(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter") void load(1);
              }}
              className="w-64"
            />
            <Button variant="outline" size="sm" onClick={() => void load(1)}>
              <Search className="size-3.5" aria-hidden />
              搜索
            </Button>
            <Select value={accountId} onValueChange={setAccountId}>
              <SelectTrigger className="w-44">
                <SelectValue placeholder="全部账号" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部账号</SelectItem>
                {accounts.map((account) => (
                  <SelectItem key={account.account_id} value={account.account_id}>
                    {account.display_name || account.account_id}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <span className="text-[length:var(--text-sm)] text-muted-foreground">
            共 {total} 件商品
          </span>
        </div>

        {/* 列表 */}
        {loading ? (
          <Loading size="lg" text="加载中..." className="py-16" />
        ) : items.length === 0 ? (
          <div className="py-16 text-center text-muted-foreground">暂无商品</div>
        ) : (
          <div className="overflow-hidden rounded-xl border border-border">
            <table className="w-full text-[length:var(--text-sm)]">
              <thead className="bg-muted/50 text-muted-foreground">
                <tr>
                  <th className="px-4 py-2.5 text-left font-medium">商品 ID</th>
                  <th className="px-4 py-2.5 text-left font-medium">标题</th>
                  <th className="px-4 py-2.5 text-left font-medium">价格</th>
                  <th className="px-4 py-2.5 text-left font-medium">标记</th>
                  <th className="px-4 py-2.5 text-left font-medium">AI 提示词</th>
                  <th className="px-4 py-2.5 text-right font-medium">操作</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border">
                {items.map((item) => (
                  <tr key={item.item_id} className="hover:bg-muted/30">
                    <td className="px-4 py-2.5 font-mono">{item.item_id}</td>
                    <td className="px-4 py-2.5">
                      <p className="max-w-64 truncate">{item.title || "—"}</p>
                      <p className="font-mono text-[length:var(--text-xs)] text-muted-foreground">
                        {item.account_id}
                      </p>
                    </td>
                    <td className="px-4 py-2.5">{formatAmount(item.price)} 元</td>
                    <td className="px-4 py-2.5">
                      <div className="flex flex-wrap gap-1">
                        {item.is_polished ? (
                          <span className="rounded-full bg-blue-500/15 px-2 py-0.5 text-[length:var(--text-xs)] text-blue-500">
                            已擦亮
                          </span>
                        ) : null}
                        {item.is_multi_spec ? (
                          <span className="rounded-full bg-amber-500/15 px-2 py-0.5 text-[length:var(--text-xs)] text-amber-500">
                            多规格
                          </span>
                        ) : null}
                        {item.has_card ? (
                          <span className="rounded-full bg-emerald-500/15 px-2 py-0.5 text-[length:var(--text-xs)] text-emerald-500">
                            有卡券
                          </span>
                        ) : null}
                      </div>
                    </td>
                    <td className="px-4 py-2.5">
                      <p className="max-w-48 truncate text-muted-foreground">
                        {item.ai_prompt || "—"}
                      </p>
                    </td>
                    <td className="px-4 py-2.5 text-right">
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => {
                          setEditTarget(item);
                          setAiPrompt(item.ai_prompt);
                        }}
                      >
                        <Pencil className="size-3.5" aria-hidden />
                        AI 提示词
                      </Button>
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

      {/* AI 提示词编辑弹窗 */}
      <ConfirmModal
        isOpen={editTarget !== null}
        title="编辑 AI 提示词"
        message={
          <div className="space-y-2 text-left">
            <p className="font-mono text-[length:var(--text-xs)] text-muted-foreground">
              {editTarget?.item_id} · {editTarget?.title}
            </p>
            <Textarea
              value={aiPrompt}
              onChange={(event) => setAiPrompt(event.target.value)}
              placeholder="商品特殊说明，如：不议价、现货直发"
              rows={4}
            />
          </div>
        }
        confirmText={saving ? "保存中…" : "保存"}
        loading={saving}
        onConfirm={() => void handleSaveAiPrompt()}
        onCancel={() => setEditTarget(null)}
      />
    </PageScaffold>
  );
}
