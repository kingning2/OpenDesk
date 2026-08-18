/**
 * 闲鱼意见反馈页（迁移自原前端 `pages/feedback/Feedback.tsx`，本地化适配版）。
 *
 * 按原前端核心交互重写：反馈列表（类型筛选）+ 新建反馈（类型/标题/内容）。
 * 数据访问走 Tauri IPC（`@desk/platform/ipc/feedback`），复用 crates/app FeedbackService。
 *
 * 说明：原前端的管理员回复 / 解决标记 / 图片上传依赖 SaaS 服务端，桌面端以本地记录管理替代。
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
import { Bug, Lightbulb, MessageSquarePlus, RefreshCw, Trash2 } from "@desk/ui/icons";
import {
  feedbackCreate,
  feedbackDelete,
  feedbackList,
  type Feedback,
  type FeedbackKind,
} from "@desk/platform/ipc/feedback";

const KIND_OPTIONS: { value: FeedbackKind; label: string }[] = [
  { value: "feature", label: "需求" },
  { value: "bug", label: "BUG" },
  { value: "other", label: "其他" },
];

const KIND_LABELS: Record<FeedbackKind, string> = {
  feature: "需求",
  bug: "BUG",
  other: "其他",
};

const KIND_ICONS: Record<FeedbackKind, React.ElementType> = {
  feature: Lightbulb,
  bug: Bug,
  other: MessageSquarePlus,
};

const PAGE_SIZE_OPTIONS = [10, 20, 50, 100];

/**
 * 闲鱼意见反馈页。
 *
 * @author agent
 * @created 2026-08-13
 */
export function XianyuFeedbackPage() {
  const [feedbacks, setFeedbacks] = useState<Feedback[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [loading, setLoading] = useState(true);
  const [filterKind, setFilterKind] = useState("");
  const [formOpen, setFormOpen] = useState(false);
  const [kind, setKind] = useState<FeedbackKind>("other");
  const [title, setTitle] = useState("");
  const [content, setContent] = useState("");
  const [saving, setSaving] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<Feedback | null>(null);
  const [deleting, setDeleting] = useState(false);

  async function load(nextPage = page, nextPageSize = pageSize) {
    try {
      const [list, count] = await feedbackList({
        page: nextPage,
        page_size: nextPageSize,
        kind: (filterKind as FeedbackKind) || undefined,
      });
      setFeedbacks(list);
      setTotal(count);
      setPage(nextPage);
      setPageSize(nextPageSize);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    let cancelled = false;
    void feedbackList({ page: 1, page_size: 20 })
      .then(([list, count]) => {
        if (cancelled) return;
        setFeedbacks(list);
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

  async function handleSubmit() {
    if (!title.trim()) {
      toast.error("请填写反馈标题");
      return;
    }
    if (!content.trim()) {
      toast.error("请填写反馈内容");
      return;
    }
    setSaving(true);
    try {
      await feedbackCreate({ kind, title: title.trim(), content: content.trim(), created_at: null });
      toast.success("反馈已提交");
      setFormOpen(false);
      setTitle("");
      setContent("");
      await load(1);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete() {
    if (!deleteTarget) return;
    setDeleting(true);
    try {
      await feedbackDelete(deleteTarget.id);
      toast.success("反馈已删除");
      setDeleteTarget(null);
      await load();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setDeleting(false);
    }
  }

  return (
    <PageScaffold subtitle="闲鱼意见反馈 — 提交需求 / BUG / 建议">
      <div className="space-y-4">
        {/* 工具栏 */}
        <div className="flex items-center justify-between gap-3">
          <div className="flex gap-2">
            <Select value={filterKind} onValueChange={setFilterKind}>
              <SelectTrigger className="w-32" aria-label="反馈类型筛选">
                <SelectValue placeholder="全部类型" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="">全部类型</SelectItem>
                {KIND_OPTIONS.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button size="sm" variant="outline" onClick={() => void load(1)}>
              <RefreshCw className="size-4" aria-hidden />
              刷新
            </Button>
          </div>
          <Button onClick={() => setFormOpen(true)}>
            <MessageSquarePlus className="size-4" aria-hidden />
            提交反馈
          </Button>
        </div>

        {/* 反馈列表 */}
        {loading ? (
          <Loading size="lg" text="加载中..." className="py-16" />
        ) : feedbacks.length === 0 ? (
          <div className="py-16 text-center text-muted-foreground">
            <MessageSquarePlus className="mx-auto mb-2 size-10 opacity-40" aria-hidden />
            暂无反馈
          </div>
        ) : (
          <div className="overflow-hidden rounded-xl border border-border">
            <div className="flex items-center justify-between border-b border-border bg-muted/30 px-4 py-2">
              <span className="text-[length:var(--text-sm)] font-medium">我的反馈</span>
              <span className="rounded-full bg-primary/15 px-2 py-0.5 text-[length:var(--text-xs)] text-primary">
                共 {total} 条
              </span>
            </div>
            <div className="max-h-[60vh] overflow-auto">
              <table className="w-full text-[length:var(--text-xs)]">
                <thead className="sticky top-0 bg-muted/80 text-muted-foreground">
                  <tr>
                    <th className="px-3 py-2 text-left font-medium">类型</th>
                    <th className="px-3 py-2 text-left font-medium">标题</th>
                    <th className="px-3 py-2 text-left font-medium">内容</th>
                    <th className="px-3 py-2 text-left font-medium">提交时间</th>
                    <th className="px-3 py-2 text-right font-medium">操作</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border">
                  {feedbacks.map((feedback) => {
                    const KindIcon = KIND_ICONS[feedback.kind];
                    return (
                      <tr key={feedback.id} className="hover:bg-muted/30">
                        <td className="px-3 py-2">
                          <span className="inline-flex items-center gap-1 rounded-full bg-muted px-2 py-0.5 text-[length:var(--text-xs)]">
                            <KindIcon className="size-3" aria-hidden />
                            {KIND_LABELS[feedback.kind]}
                          </span>
                        </td>
                        <td className="max-w-48 px-3 py-2 font-medium">{feedback.title}</td>
                        <td className="max-w-64 px-3 py-2 text-muted-foreground">
                          <span className="line-clamp-2">{feedback.content}</span>
                        </td>
                        <td className="whitespace-nowrap px-3 py-2 text-muted-foreground">
                          {feedback.created_at ?? "-"}
                        </td>
                        <td className="px-3 py-2 text-right">
                          <Button
                            size="sm"
                            variant="ghost"
                            className="text-destructive"
                            onClick={() => setDeleteTarget(feedback)}
                          >
                            <Trash2 className="size-3.5" aria-hidden />
                          </Button>
                        </td>
                      </tr>
                    );
                  })}
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

      {/* 新建反馈弹窗 */}
      <ConfirmModal
        isOpen={formOpen}
        title="提交反馈"
        message={
          <div className="space-y-3 text-left">
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">反馈类型</span>
              <Select value={kind} onValueChange={(value) => setKind(value as FeedbackKind)}>
                <SelectTrigger aria-label="反馈类型">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {KIND_OPTIONS.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">标题 *</span>
              <Input
                value={title}
                onChange={(event) => setTitle(event.target.value)}
                placeholder="反馈标题"
              />
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">内容 *</span>
              <Textarea
                value={content}
                onChange={(event) => setContent(event.target.value)}
                rows={4}
                placeholder="详细描述您的需求、问题或建议"
              />
            </label>
          </div>
        }
        confirmText={saving ? "提交中…" : "提交"}
        loading={saving}
        onConfirm={() => void handleSubmit()}
        onCancel={() => {
          setFormOpen(false);
          setTitle("");
          setContent("");
        }}
      />

      {/* 删除确认 */}
      <ConfirmModal
        isOpen={deleteTarget !== null}
        type="danger"
        title="删除反馈"
        message={`确认删除反馈「${deleteTarget?.title ?? ""}」？`}
        confirmText="删除"
        loading={deleting}
        onConfirm={() => void handleDelete()}
        onCancel={() => setDeleteTarget(null)}
      />
    </PageScaffold>
  );
}
