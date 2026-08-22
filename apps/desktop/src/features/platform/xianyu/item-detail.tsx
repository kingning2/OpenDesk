/**
 * 闲鱼商品详情页 — 拉取平台详情并展示图文。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */

import { OWNER_ID } from "@desk/platform/constants";
import { useEffect, useState } from "react";
import {
  Button,
  ConfirmModal,
  Loading,
  PageScaffold,
  Textarea,
  toast,
} from "@desk/ui";
import { ArrowLeft, ExternalLink, Pencil } from "@desk/ui/icons";
import { managePath } from "@desk/platform/compile";
import {
  itemDetailFetch,
  itemGet,
  itemUpdate,
  type ItemDetail,
} from "@desk/platform/ipc/item";
import { useWorkspaceNav } from "../../../app/use-workspace-tabs";
import { formatAmount } from "@desk/utils";


export interface XianyuItemDetailPageProps {
  /** 闲鱼商品 ID。 */
  itemId: string;
}

/**
 * 闲鱼商品详情页。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export function XianyuItemDetailPage({ itemId }: XianyuItemDetailPageProps) {
  const { selectTab } = useWorkspaceNav();
  const [detail, setDetail] = useState<ItemDetail | null>(null);
  const [aiPrompt, setAiPrompt] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [editOpen, setEditOpen] = useState(false);
  const [draftPrompt, setDraftPrompt] = useState("");

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    void Promise.all([itemDetailFetch(OWNER_ID, itemId), itemGet(OWNER_ID, itemId)])
      .then(([fetched, local]) => {
        if (cancelled) return;
        setDetail(fetched);
        setAiPrompt(local?.ai_prompt ?? "");
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
  }, [itemId]);

  async function handleSaveAiPrompt() {
    setSaving(true);
    try {
      await itemUpdate(OWNER_ID, itemId, draftPrompt);
      setAiPrompt(draftPrompt);
      setEditOpen(false);
      toast.success("AI 提示词已保存");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

  return (
    <PageScaffold subtitle="闲鱼商品详情">
      <div className="space-y-4">
        <div className="flex flex-wrap items-center gap-2">
          <Button variant="ghost" size="sm" onClick={() => selectTab(managePath("items"))}>
            <ArrowLeft className="size-3.5" aria-hidden />
            返回列表
          </Button>
          {detail ? (
            <Button variant="outline" size="sm" asChild>
              <a href={detail.item_url} target="_blank" rel="noreferrer">
                <ExternalLink className="size-3.5" aria-hidden />
                在闲鱼打开
              </a>
            </Button>
          ) : null}
        </div>

        {loading ? (
          <Loading size="lg" text="加载商品详情..." className="py-16" />
        ) : !detail ? (
          <div className="py-16 text-center text-muted-foreground">无法加载商品详情</div>
        ) : (
          <div className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.1fr)]">
            {/* 图片区 */}
            <div className="space-y-3">
              {detail.images.length > 0 ? (
                <>
                  <div className="overflow-hidden rounded-2xl border border-border bg-muted/30">
                    <img
                      src={detail.images[0]}
                      alt={detail.title || "商品主图"}
                      className="aspect-square w-full object-contain"
                    />
                  </div>
                  {detail.images.length > 1 ? (
                    <div className="grid grid-cols-4 gap-2 sm:grid-cols-5">
                      {detail.images.slice(1).map((url) => (
                        <div
                          key={url}
                          className="overflow-hidden rounded-lg border border-border bg-muted/20"
                        >
                          <img src={url} alt="" className="aspect-square w-full object-cover" />
                        </div>
                      ))}
                    </div>
                  ) : null}
                </>
              ) : (
                <div className="flex aspect-square items-center justify-center rounded-2xl border border-dashed border-border text-muted-foreground">
                  暂无图片
                </div>
              )}
            </div>

            {/* 文案区 */}
            <div className="space-y-4">
              <div className="rounded-2xl border border-border bg-card p-5">
                <h1 className="text-[length:var(--text-xl)] font-semibold leading-snug">
                  {detail.title || "—"}
                </h1>
                <p className="mt-2 font-mono text-[length:var(--text-xs)] text-muted-foreground">
                  {detail.item_id}
                </p>
                <div className="mt-4 flex flex-wrap items-baseline gap-3">
                  <span className="text-[length:var(--text-2xl)] font-semibold text-primary">
                    {formatAmount(detail.price)} 元
                  </span>
                  {detail.original_price != null && detail.original_price > detail.price ? (
                    <span className="text-[length:var(--text-sm)] text-muted-foreground line-through">
                      {formatAmount(detail.original_price)} 元
                    </span>
                  ) : null}
                </div>
                <div className="mt-3 flex flex-wrap gap-3 text-[length:var(--text-sm)] text-muted-foreground">
                  {detail.want_count != null ? <span>{detail.want_count} 人想要</span> : null}
                  {detail.browse_count != null ? <span>{detail.browse_count} 次浏览</span> : null}
                </div>
              </div>

              <div className="rounded-2xl border border-border bg-card p-5">
                <h2 className="text-[length:var(--text-sm)] font-medium text-muted-foreground">
                  商品描述
                </h2>
                <p className="mt-3 whitespace-pre-wrap text-[length:var(--text-sm)] leading-relaxed">
                  {detail.desc || "暂无描述"}
                </p>
              </div>

              <div className="rounded-2xl border border-border bg-card p-5">
                <div className="flex items-center justify-between gap-3">
                  <h2 className="text-[length:var(--text-sm)] font-medium text-muted-foreground">
                    AI 提示词
                  </h2>
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => {
                      setDraftPrompt(aiPrompt);
                      setEditOpen(true);
                    }}
                  >
                    <Pencil className="size-3.5" aria-hidden />
                    编辑
                  </Button>
                </div>
                <p className="mt-3 whitespace-pre-wrap text-[length:var(--text-sm)] text-muted-foreground">
                  {aiPrompt || "—"}
                </p>
              </div>
            </div>
          </div>
        )}
      </div>

      <ConfirmModal
        isOpen={editOpen}
        title="编辑 AI 提示词"
        message={
          <Textarea
            value={draftPrompt}
            onChange={(event) => setDraftPrompt(event.target.value)}
            placeholder="商品特殊说明，如：不议价、现货直发"
            rows={4}
          />
        }
        confirmText={saving ? "保存中…" : "保存"}
        loading={saving}
        onConfirm={() => void handleSaveAiPrompt()}
        onCancel={() => setEditOpen(false)}
      />
    </PageScaffold>
  );
}
