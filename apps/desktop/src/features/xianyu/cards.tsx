/**
 * 闲鱼卡券管理页（迁移自原前端 `pages/cards/Cards.tsx`）。
 *
 * 按原前端核心交互重写：卡券列表 + 类型筛选 + 新建/编辑（text/data/api/image 四种类型）+ 启用切换 + 删除。
 * 数据访问走 Tauri IPC（`@desk/platform/ipc/card`），复用 crates/app CardService。
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
import { Pencil, Plus, Trash2 } from "@desk/ui/icons";
import {
  cardCreate,
  cardDelete,
  cardList,
  cardSetEnabled,
  cardUpdate,
  type Card,
} from "@desk/platform/ipc/card";

const OWNER_ID = 1; // 桌面单用户；多用户时由登录态注入
const PAGE_SIZE = 20;

const TYPE_OPTIONS = [
  { value: "", label: "全部类型" },
  { value: "text", label: "文本" },
  { value: "data", label: "批量数据" },
  { value: "api", label: "API 拉取" },
  { value: "image", label: "图片" },
];

const TYPE_LABEL: Record<string, string> = {
  text: "文本",
  data: "批量数据",
  api: "API 拉取",
  image: "图片",
};

interface CardForm {
  name: string;
  card_type: string;
  text_content: string;
  data_content: string;
  image_url: string;
  description: string;
  delay_seconds: number;
}

const EMPTY_FORM: CardForm = {
  name: "",
  card_type: "text",
  text_content: "",
  data_content: "",
  image_url: "",
  description: "",
  delay_seconds: 0,
};

function formToCard(form: CardForm, existing?: Card): Card {
  return {
    id: existing?.id ?? 0,
    owner_id: existing?.owner_id ?? OWNER_ID,
    account_id: existing?.account_id ?? "",
    name: form.name.trim(),
    card_type: form.card_type,
    source: existing?.source ?? "own",
    enabled: existing?.enabled ?? true,
    text_content: form.text_content,
    data_content: form.data_content,
    image_url: form.image_url,
    image_urls: existing?.image_urls ?? "",
    api_config: existing?.api_config ?? "",
    delay_seconds: form.delay_seconds,
    description: form.description,
  };
}

/**
 * 闲鱼卡券管理页。
 *
 * @author agent
 * @created 2026-08-13
 */
export function XianyuCardsPage() {
  const [cards, setCards] = useState<Card[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [loading, setLoading] = useState(false);
  const [keyword, setKeyword] = useState("");
  const [typeFilter, setTypeFilter] = useState("");
  const [showForm, setShowForm] = useState(false);
  const [editTarget, setEditTarget] = useState<Card | null>(null);
  const [form, setForm] = useState<CardForm>(EMPTY_FORM);
  const [deleteTarget, setDeleteTarget] = useState<Card | null>(null);
  const [saving, setSaving] = useState(false);

  async function load(nextPage = page, nextType = typeFilter) {
    setLoading(true);
    try {
      const [list, count] = await cardList({
        owner_id: OWNER_ID,
        page: nextPage,
        page_size: PAGE_SIZE,
        keyword: keyword.trim() || undefined,
        card_type: nextType || undefined,
      });
      setCards(list);
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
    void cardList({
      owner_id: OWNER_ID,
      page: 1,
      page_size: PAGE_SIZE,
      keyword: keyword.trim() || undefined,
      card_type: typeFilter || undefined,
    })
      .then(([list, count]) => {
        if (cancelled) return;
        setCards(list);
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
  }, [typeFilter]);

  const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE));

  function openCreate() {
    setEditTarget(null);
    setForm(EMPTY_FORM);
    setShowForm(true);
  }

  function openEdit(card: Card) {
    setEditTarget(card);
    setForm({
      name: card.name,
      card_type: card.card_type,
      text_content: card.text_content,
      data_content: card.data_content,
      image_url: card.image_url,
      description: card.description,
      delay_seconds: card.delay_seconds,
    });
    setShowForm(true);
  }

  async function handleSave() {
    if (!form.name.trim()) {
      toast.error("卡券名称不能为空");
      return;
    }
    setSaving(true);
    try {
      if (editTarget) {
        await cardUpdate(OWNER_ID, formToCard(form, editTarget));
        toast.success("卡券已更新");
      } else {
        await cardCreate(OWNER_ID, formToCard(form));
        toast.success("卡券已创建");
      }
      setShowForm(false);
      await load();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

  async function handleToggleEnabled(card: Card) {
    try {
      await cardSetEnabled(OWNER_ID, card.id, !card.enabled);
      toast.success(card.enabled ? "卡券已停用" : "卡券已启用");
      await load();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  async function handleDelete() {
    if (!deleteTarget) return;
    try {
      await cardDelete(OWNER_ID, deleteTarget.id);
      toast.success("卡券已删除");
      setDeleteTarget(null);
      await load();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  return (
    <PageScaffold subtitle="闲鱼卡券管理 — 自动发货内容库">
      <div className="space-y-4">
        {/* 工具栏 */}
        <div className="flex items-center justify-between gap-3">
          <div className="flex items-center gap-3">
            <Input
              placeholder="搜索卡券名称"
              value={keyword}
              onChange={(event) => setKeyword(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter") void load(1);
              }}
              className="w-56"
            />
            <Select value={typeFilter} onValueChange={setTypeFilter}>
              <SelectTrigger className="w-32">
                <SelectValue placeholder="全部类型" />
              </SelectTrigger>
              <SelectContent>
                {TYPE_OPTIONS.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <Button onClick={openCreate}>
            <Plus className="size-4" aria-hidden />
            新增卡券
          </Button>
        </div>

        {/* 列表 */}
        {loading ? (
          <Loading size="lg" text="加载中..." className="py-16" />
        ) : cards.length === 0 ? (
          <div className="py-16 text-center text-muted-foreground">暂无卡券</div>
        ) : (
          <div className="overflow-hidden rounded-xl border border-border">
            <table className="w-full text-[length:var(--text-sm)]">
              <thead className="bg-muted/50 text-muted-foreground">
                <tr>
                  <th className="px-4 py-2.5 text-left font-medium">名称</th>
                  <th className="px-4 py-2.5 text-left font-medium">类型</th>
                  <th className="px-4 py-2.5 text-left font-medium">内容预览</th>
                  <th className="px-4 py-2.5 text-left font-medium">延时</th>
                  <th className="px-4 py-2.5 text-left font-medium">状态</th>
                  <th className="px-4 py-2.5 text-right font-medium">操作</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border">
                {cards.map((card) => (
                  <tr key={card.id} className="hover:bg-muted/30">
                    <td className="px-4 py-2.5 font-medium">{card.name}</td>
                    <td className="px-4 py-2.5">
                      <span className="rounded-full bg-muted px-2 py-0.5 text-[length:var(--text-xs)]">
                        {TYPE_LABEL[card.card_type] ?? card.card_type}
                      </span>
                    </td>
                    <td className="px-4 py-2.5">
                      <p className="max-w-56 truncate text-muted-foreground">
                        {card.text_content || card.data_content.split("\n")[0] || card.image_url || "—"}
                      </p>
                    </td>
                    <td className="px-4 py-2.5 text-muted-foreground">
                      {card.delay_seconds > 0 ? `${card.delay_seconds}s` : "—"}
                    </td>
                    <td className="px-4 py-2.5">
                      <span
                        className={
                          card.enabled
                            ? "rounded-full bg-emerald-500/15 px-2 py-0.5 text-[length:var(--text-xs)] text-emerald-500"
                            : "rounded-full bg-muted px-2 py-0.5 text-[length:var(--text-xs)] text-muted-foreground"
                        }
                      >
                        {card.enabled ? "启用" : "停用"}
                      </span>
                    </td>
                    <td className="px-4 py-2.5 text-right">
                      <div className="flex items-center justify-end gap-1">
                        <Button size="sm" variant="outline" onClick={() => void handleToggleEnabled(card)}>
                          {card.enabled ? "停用" : "启用"}
                        </Button>
                        <Button size="sm" variant="ghost" onClick={() => openEdit(card)}>
                          <Pencil className="size-3.5" aria-hidden />
                        </Button>
                        <Button
                          size="sm"
                          variant="ghost"
                          className="text-destructive"
                          onClick={() => setDeleteTarget(card)}
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

      {/* 新建/编辑弹窗 */}
      <ConfirmModal
        isOpen={showForm}
        title={editTarget ? "编辑卡券" : "新增卡券"}
        message={
          <div className="space-y-3 text-left">
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">名称</span>
              <Input
                value={form.name}
                onChange={(event) => setForm({ ...form, name: event.target.value })}
                placeholder="如：会员激活码"
              />
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">类型</span>
              <Select
                value={form.card_type}
                onValueChange={(value) => setForm({ ...form, card_type: value })}
              >
                <SelectTrigger className="w-full">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {TYPE_OPTIONS.filter((option) => option.value).map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </label>
            {form.card_type === "text" ? (
              <label className="block space-y-1">
                <span className="text-[length:var(--text-sm)] text-muted-foreground">文本内容</span>
                <Textarea
                  value={form.text_content}
                  onChange={(event) => setForm({ ...form, text_content: event.target.value })}
                  placeholder="发货内容"
                  rows={3}
                />
              </label>
            ) : null}
            {form.card_type === "data" ? (
              <label className="block space-y-1">
                <span className="text-[length:var(--text-sm)] text-muted-foreground">
                  批量数据（每行一条）
                </span>
                <Textarea
                  value={form.data_content}
                  onChange={(event) => setForm({ ...form, data_content: event.target.value })}
                  placeholder={"卡密1\n卡密2"}
                  rows={4}
                />
              </label>
            ) : null}
            {form.card_type === "image" ? (
              <label className="block space-y-1">
                <span className="text-[length:var(--text-sm)] text-muted-foreground">图片 URL</span>
                <Input
                  value={form.image_url}
                  onChange={(event) => setForm({ ...form, image_url: event.target.value })}
                  placeholder="https://..."
                />
              </label>
            ) : null}
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">备注</span>
              <Input
                value={form.description}
                onChange={(event) => setForm({ ...form, description: event.target.value })}
                placeholder="可选"
              />
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">发货延时（秒）</span>
              <Input
                type="number"
                min={0}
                value={form.delay_seconds}
                onChange={(event) =>
                  setForm({ ...form, delay_seconds: Number(event.target.value) || 0 })
                }
              />
            </label>
          </div>
        }
        confirmText={saving ? "保存中…" : "保存"}
        loading={saving}
        onConfirm={() => void handleSave()}
        onCancel={() => setShowForm(false)}
      />

      {/* 删除确认 */}
      <ConfirmModal
        isOpen={deleteTarget !== null}
        type="danger"
        title="删除卡券"
        message={`确认删除卡券「${deleteTarget?.name ?? ""}」？`}
        confirmText="删除"
        onConfirm={() => void handleDelete()}
        onCancel={() => setDeleteTarget(null)}
      />
    </PageScaffold>
  );
}
