/**
 * 闲鱼自动回复关键词页（迁移自原前端 `pages/keywords/Keywords.tsx`）。
 *
 * 按原前端核心交互重写：按账号查看关键词 → 新增 / 编辑 / 删除 / 整表保存。
 * 数据访问走 Tauri IPC（`@desk/platform/ipc/keyword`），复用 crates/app KeywordService。
 */

import { useEffect, useMemo, useState } from "react";
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
import {
  keywordAdd,
  keywordDelete,
  keywordList,
  keywordReplace,
  type KeywordRule,
} from "@desk/platform/ipc/keyword";
import { accountList, type XianyuAccount } from "@desk/platform/ipc/account";
import { Check, Plus, Trash2 } from "@desk/ui/icons";

const OWNER_ID = 1; // 桌面单用户；多用户时由登录态注入

interface KeywordForm {
  keyword: string;
  reply: string;
  item_id: string;
}

const EMPTY_FORM: KeywordForm = { keyword: "", reply: "", item_id: "" };

function toRule(accountId: string, form: KeywordForm): KeywordRule {
  return {
    id: 0,
    account_id: accountId,
    keyword: form.keyword.trim(),
    reply: form.reply,
    item_id: form.item_id.trim(),
    rule_type: "text",
    image_url: "",
    item_title: "",
  };
}

/**
 * 闲鱼自动回复关键词页。
 *
 * @author agent
 * @created 2026-08-13
 */
export function XianyuKeywordsPage() {
  const [accounts, setAccounts] = useState<XianyuAccount[]>([]);
  const [accountId, setAccountId] = useState("");
  const [rules, setRules] = useState<KeywordRule[]>([]);
  const [loading, setLoading] = useState(false);
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState<KeywordForm>(EMPTY_FORM);
  const [deleteTarget, setDeleteTarget] = useState<KeywordRule | null>(null);
  const [saving, setSaving] = useState(false);

  // 初始加载账号列表。
  useEffect(() => {
    let cancelled = false;
    void accountList(OWNER_ID)
      .then((list) => {
        if (cancelled) return;
        setAccounts(list);
        setAccountId(list[0]?.account_id ?? "");
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

  // 按选中账号加载关键词。
  useEffect(() => {
    if (!accountId) return;
    let cancelled = false;
    void keywordList(accountId)
      .then((list) => {
        if (!cancelled) {
          setRules(list);
        }
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

  const grouped = useMemo(() => {
    const global = rules.filter((rule) => !rule.item_id);
    const byItem = new Map<string, KeywordRule[]>();
    for (const rule of rules) {
      if (!rule.item_id) continue;
      const list = byItem.get(rule.item_id) ?? [];
      list.push(rule);
      byItem.set(rule.item_id, list);
    }
    return { global, byItem };
  }, [rules]);

  async function handleSaveAll() {
    setSaving(true);
    try {
      await keywordReplace(accountId, rules);
      toast.success("关键词已保存");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

  async function handleAdd() {
    if (!form.keyword.trim()) {
      toast.error("关键词不能为空");
      return;
    }
    setSaving(true);
    try {
      const created = await keywordAdd(accountId, toRule(accountId, form));
      setRules((current) => [...current, created]);
      toast.success("关键词已添加");
      setShowForm(false);
      setForm(EMPTY_FORM);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete() {
    if (!deleteTarget) return;
    try {
      await keywordDelete(deleteTarget.id);
      setRules((current) => current.filter((rule) => rule.id !== deleteTarget.id));
      toast.success("关键词已删除");
      setDeleteTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  function KeywordRow({ rule }: { rule: KeywordRule }) {
    return (
      <div className="flex items-center gap-3 rounded-[var(--radius-md)] border border-border/70 px-3 py-2">
        <span className="w-40 shrink-0 truncate font-mono text-[length:var(--text-sm)] text-primary">
          {rule.keyword.split("\n")[0]}
        </span>
        <span className="min-w-0 flex-1 truncate text-[length:var(--text-sm)] text-muted-foreground">
          {rule.reply || "—"}
        </span>
        <Button
          size="sm"
          variant="ghost"
          className="shrink-0 text-destructive"
          onClick={() => setDeleteTarget(rule)}
        >
          <Trash2 className="size-3.5" aria-hidden />
        </Button>
      </div>
    );
  }

  return (
    <PageScaffold subtitle="闲鱼自动回复 — 关键词规则管理">
      <div className="space-y-4">
        {/* 工具栏 */}
        <div className="flex items-center justify-between gap-3">
          <div className="flex items-center gap-3">
            <Select value={accountId} onValueChange={setAccountId}>
              <SelectTrigger className="w-48">
                <SelectValue placeholder="选择账号" />
              </SelectTrigger>
              <SelectContent>
                {accounts.map((account) => (
                  <SelectItem key={account.account_id} value={account.account_id}>
                    {account.display_name || account.account_id}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button variant="outline" size="sm" onClick={() => void handleSaveAll()} disabled={saving}>
              <Check className="size-3.5" aria-hidden />
              保存全部
            </Button>
          </div>
          <Button onClick={() => setShowForm(true)}>
            <Plus className="size-4" aria-hidden />
            新增关键词
          </Button>
        </div>

        {/* 列表 */}
        {loading ? (
          <Loading size="lg" text="加载中..." className="py-16" />
        ) : rules.length === 0 ? (
          <div className="py-16 text-center text-muted-foreground">
            暂无关键词，点击「新增关键词」添加
          </div>
        ) : (
          <div className="space-y-4">
            {grouped.global.length > 0 ? (
              <section className="space-y-2">
                <h3 className="text-[length:var(--text-sm)] font-semibold text-muted-foreground">
                  全局关键词
                </h3>
                {grouped.global.map((rule) => (
                  <KeywordRow key={rule.id} rule={rule} />
                ))}
              </section>
            ) : null}
            {Array.from(grouped.byItem.entries()).map(([itemId, itemRules]) => (
              <section key={itemId} className="space-y-2">
                <h3 className="text-[length:var(--text-sm)] font-semibold text-muted-foreground">
                  商品 {itemId}
                </h3>
                {itemRules.map((rule) => (
                  <KeywordRow key={rule.id} rule={rule} />
                ))}
              </section>
            ))}
          </div>
        )}
      </div>

      {/* 新增关键词弹窗 */}
      <ConfirmModal
        isOpen={showForm}
        title="新增关键词"
        message={
          <div className="space-y-3 text-left">
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">
                关键词（多行 = 多个触发词）
              </span>
              <Textarea
                value={form.keyword}
                onChange={(event) => setForm({ ...form, keyword: event.target.value })}
                placeholder={"在吗\n你好"}
                rows={3}
              />
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">
                回复内容（支持 {"{send_user_name}"} 等变量）
              </span>
              <Textarea
                value={form.reply}
                onChange={(event) => setForm({ ...form, reply: event.target.value })}
                placeholder="在的，请问需要什么帮助？"
                rows={3}
              />
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">
                关联商品 ID（留空 = 全局）
              </span>
              <Input
                value={form.item_id}
                onChange={(event) => setForm({ ...form, item_id: event.target.value })}
                placeholder="可选"
              />
            </label>
          </div>
        }
        confirmText={saving ? "添加中…" : "添加"}
        loading={saving}
        onConfirm={() => void handleAdd()}
        onCancel={() => {
          setShowForm(false);
          setForm(EMPTY_FORM);
        }}
      />

      {/* 删除确认 */}
      <ConfirmModal
        isOpen={deleteTarget !== null}
        type="danger"
        title="删除关键词"
        message={`确认删除关键词「${deleteTarget?.keyword.split("\n")[0] ?? ""}」？`}
        confirmText="删除"
        onConfirm={() => void handleDelete()}
        onCancel={() => setDeleteTarget(null)}
      />
    </PageScaffold>
  );
}
