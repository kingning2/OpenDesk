/**
 * 闲鱼单品发布页（迁移自原前端 `pages/product-publish/ProductPublish.tsx`）。
 *
 * 按原前端核心交互重写：账号选择 + 账号发布能力检测 + 商品核心字段表单 +
 * 从素材库导入 + 立即发布（结果展示：成功跳转查看商品 / 失败原因）。
 * 数据访问走 Tauri IPC（`@desk/platform/ipc/publish`），复用 crates/app PublishService 编排。
 *
 * 说明：内存网关模拟发布成功（实际平台发布由 sidecar 执行）；
 * 规格/SKU/视频/平台属性编辑器依赖外部服务，桌面端以核心字段表单替代。
 */

import { useEffect, useState } from "react";
import {
  Button,
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
import { CheckCircle, ExternalLink, FolderOpen, Send, XCircle } from "@desk/ui/icons";
import { accountList, type XianyuAccount } from "@desk/platform/ipc/account";
import {
  publishCapability,
  publishSingle,
  type PublishAccountCapability,
  type PublishResult,
} from "@desk/platform/ipc/publish";
import { publishMaterialList, type PublishMaterial } from "@desk/platform/ipc/publish-material";

const OWNER_ID = 1; // 桌面单用户；多用户时由登录态注入
const CONDITIONS = ["全新", "99新", "95新", "9成新", "8成新", "7成新以下"];

/** 商品表单（核心字段）。 */
interface PublishForm {
  accountId: string;
  title: string;
  description: string;
  price: string;
  originalPrice: string;
  category: string;
  condition: string;
  quantity: string;
  deliveryMethod: string;
  shippingMethod: string;
  postage: string;
  brand: string;
  images: string[];
}

const EMPTY_FORM: PublishForm = {
  accountId: "",
  title: "",
  description: "",
  price: "",
  originalPrice: "",
  category: "",
  condition: "全新",
  quantity: "1",
  deliveryMethod: "express",
  shippingMethod: "free",
  postage: "0",
  brand: "",
  images: [],
};

/**
 * 闲鱼单品发布页。
 *
 * @author agent
 * @created 2026-08-13
 */
export function XianyuProductPublishPage() {
  const [accounts, setAccounts] = useState<XianyuAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [capability, setCapability] = useState<PublishAccountCapability | null>(null);
  const [capabilityLoading, setCapabilityLoading] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [result, setResult] = useState<PublishResult | null>(null);
  const [showPicker, setShowPicker] = useState(false);
  const [materials, setMaterials] = useState<PublishMaterial[]>([]);
  const [form, setForm] = useState<PublishForm>(EMPTY_FORM);

  const patchForm = (patch: Partial<PublishForm>) =>
    setForm((current) => ({ ...current, ...patch }));

  // 加载账号（默认第一个启用账号）。
  useEffect(() => {
    let cancelled = false;
    void accountList(OWNER_ID)
      .then((list) => {
        if (cancelled) return;
        setAccounts(list);
        const defaultAccount = list.find((account) => account.status === "active") ?? list[0];
        if (defaultAccount) {
          setForm((current) => ({ ...current, accountId: defaultAccount.account_id }));
        }
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

  // 账号切换 → 能力检测。
  useEffect(() => {
    if (!form.accountId) return;
    let cancelled = false;
    void publishCapability(form.accountId)
      .then((result) => {
        if (cancelled) return;
        setCapability(result);
        if (!result.success) {
          toast.error(result.message);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          toast.error(error instanceof Error ? error.message : String(error));
        }
      })
      .finally(() => {
        if (!cancelled) setCapabilityLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [form.accountId]);

  async function loadMaterials() {
    try {
      const [list] = await publishMaterialList({ page: 1, page_size: 100 });
      setMaterials(list);
      setShowPicker(true);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    }
  }

  function applyMaterial(material: PublishMaterial) {
    let images: string[] = [];
    try {
      const parsed = JSON.parse(material.images) as unknown;
      if (Array.isArray(parsed)) images = parsed.filter((item): item is string => typeof item === "string");
    } catch {
      images = [];
    }
    patchForm({
      title: material.title,
      price: String(material.price),
      category: material.category ?? "",
      condition: material.condition,
      quantity: String(material.quantity),
      deliveryMethod: material.delivery_method,
      shippingMethod: material.shipping_method,
      postage: String(material.postage),
      images,
    });
    setShowPicker(false);
    toast.success("已从素材库导入");
  }

  async function handlePublish() {
    if (!form.accountId) {
      toast.error("请选择发布账号");
      return;
    }
    if (!form.title.trim()) {
      toast.error("请填写商品标题");
      return;
    }
    const price = Number(form.price);
    if (!Number.isFinite(price) || price <= 0) {
      toast.error("请填写有效价格");
      return;
    }
    if (form.images.length === 0) {
      toast.error("请至少填写一张商品图片 URL");
      return;
    }
    setSubmitting(true);
    setResult(null);
    try {
      const response = await publishSingle(form.accountId, {
        title: form.title.trim(),
        description: form.description,
        price,
        original_price: form.originalPrice.trim() ? Number(form.originalPrice) : undefined,
        category: form.category.trim() || undefined,
        condition: form.condition,
        quantity: Math.max(1, Number(form.quantity) || 1),
        delivery_method: form.deliveryMethod,
        shipping_method: form.shippingMethod,
        postage: Number(form.postage) || 0,
        brand: form.brand.trim() || undefined,
        images: form.images,
      });
      setResult(response);
      toast[response.success ? "success" : "error"](response.message);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <PageScaffold subtitle="闲鱼单品发布 — 填写商品信息并发布">
      <div className="space-y-4">
        {/* 工具栏 */}
        <div className="flex items-center justify-between gap-3">
          <h1 className="font-semibold">单品发布</h1>
          <Button variant="outline" size="sm" onClick={() => void loadMaterials()}>
            <FolderOpen className="size-4" aria-hidden />
            从素材库导入
          </Button>
        </div>

        {loading ? (
          <Loading size="lg" text="加载中..." className="py-16" />
        ) : (
          <div className="grid grid-cols-1 gap-4 xl:grid-cols-3">
            {/* 表单区 */}
            <div className="space-y-4 xl:col-span-2">
              <div className="rounded-xl border border-border bg-shell p-4">
                <h2 className="mb-3 font-medium">商品信息</h2>
                <div className="space-y-3">
                  <label className="block space-y-1">
                    <span className="text-[length:var(--text-xs)] text-muted-foreground">发布账号</span>
                    <Select
                      value={form.accountId}
                      onValueChange={(value) => {
                        setCapabilityLoading(true);
                        patchForm({ accountId: value });
                      }}
                    >
                      <SelectTrigger aria-label="发布账号">
                        <SelectValue placeholder="请选择发布账号" />
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
                    {capability ? (
                      <p
                        className={
                          capability.success
                            ? "text-[length:var(--text-xs)] text-emerald-600"
                            : "text-[length:var(--text-xs)] text-red-600"
                        }
                      >
                        {capabilityLoading ? "检测账号能力中..." : capability.message}
                      </p>
                    ) : null}
                  </label>
                  <label className="block space-y-1">
                    <span className="text-[length:var(--text-xs)] text-muted-foreground">商品标题 *</span>
                    <Input
                      value={form.title}
                      onChange={(event) => patchForm({ title: event.target.value })}
                      placeholder="商品标题"
                    />
                  </label>
                  <label className="block space-y-1">
                    <span className="text-[length:var(--text-xs)] text-muted-foreground">商品描述 *</span>
                    <Textarea
                      value={form.description}
                      onChange={(event) => patchForm({ description: event.target.value })}
                      rows={4}
                      placeholder="商品描述（不超过1500字）"
                    />
                  </label>
                  <div className="grid grid-cols-2 gap-3">
                    <label className="block space-y-1">
                      <span className="text-[length:var(--text-xs)] text-muted-foreground">价格（元）*</span>
                      <Input
                        type="number"
                        min={0}
                        step="0.01"
                        value={form.price}
                        onChange={(event) => patchForm({ price: event.target.value })}
                      />
                    </label>
                    <label className="block space-y-1">
                      <span className="text-[length:var(--text-xs)] text-muted-foreground">原价（元）</span>
                      <Input
                        type="number"
                        min={0}
                        step="0.01"
                        value={form.originalPrice}
                        onChange={(event) => patchForm({ originalPrice: event.target.value })}
                      />
                    </label>
                  </div>
                  <div className="grid grid-cols-2 gap-3">
                    <label className="block space-y-1">
                      <span className="text-[length:var(--text-xs)] text-muted-foreground">分类</span>
                      <Input
                        value={form.category}
                        onChange={(event) => patchForm({ category: event.target.value })}
                        placeholder="如：数码"
                      />
                    </label>
                    <label className="block space-y-1">
                      <span className="text-[length:var(--text-xs)] text-muted-foreground">成色</span>
                      <Select
                        value={form.condition}
                        onValueChange={(value) => patchForm({ condition: value })}
                      >
                        <SelectTrigger aria-label="成色">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          {CONDITIONS.map((condition) => (
                            <SelectItem key={condition} value={condition}>
                              {condition}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </label>
                  </div>
                  <div className="grid grid-cols-3 gap-3">
                    <label className="block space-y-1">
                      <span className="text-[length:var(--text-xs)] text-muted-foreground">数量</span>
                      <Input
                        type="number"
                        min={1}
                        value={form.quantity}
                        onChange={(event) => patchForm({ quantity: event.target.value })}
                      />
                    </label>
                    <label className="block space-y-1">
                      <span className="text-[length:var(--text-xs)] text-muted-foreground">邮费（元）</span>
                      <Input
                        type="number"
                        min={0}
                        step="0.01"
                        value={form.postage}
                        onChange={(event) => patchForm({ postage: event.target.value })}
                      />
                    </label>
                    <label className="block space-y-1">
                      <span className="text-[length:var(--text-xs)] text-muted-foreground">品牌</span>
                      <Input
                        value={form.brand}
                        onChange={(event) => patchForm({ brand: event.target.value })}
                        placeholder="品牌"
                      />
                    </label>
                  </div>
                  <label className="block space-y-1">
                    <span className="text-[length:var(--text-xs)] text-muted-foreground">
                      商品图片 URL（每行一个，首个为封面）*
                    </span>
                    <Textarea
                      value={form.images.join("\n")}
                      onChange={(event) =>
                        patchForm({
                          images: event.target.value
                            .split("\n")
                            .map((url) => url.trim())
                            .filter(Boolean),
                        })
                      }
                      rows={3}
                      className="font-mono text-[length:var(--text-xs)]"
                      placeholder={"https://example.com/cover.jpg\nhttps://example.com/2.jpg"}
                    />
                  </label>
                </div>
              </div>
            </div>

            {/* 操作区 */}
            <div className="space-y-4">
              <div className="rounded-xl border border-border bg-shell p-4">
                <Button
                  className="w-full"
                  disabled={submitting || capabilityLoading || !capability?.success}
                  onClick={() => void handlePublish()}
                >
                  {submitting ? (
                    "发布中..."
                  ) : capabilityLoading ? (
                    "检测账号能力中..."
                  ) : (
                    <>
                      <Send className="size-4" aria-hidden />
                      立即发布
                    </>
                  )}
                </Button>
                {submitting ? (
                  <p className="mt-2 text-center text-[length:var(--text-xs)] text-muted-foreground">
                    发布处理中，请勿重复提交
                  </p>
                ) : null}
              </div>

              {result ? (
                <div
                  className={`rounded-xl border-l-4 bg-shell p-4 ${
                    result.success ? "border-l-emerald-500" : "border-l-red-500"
                  }`}
                >
                  <div className="flex items-start gap-3">
                    {result.success ? (
                      <CheckCircle className="mt-0.5 size-5 shrink-0 text-emerald-600" aria-hidden />
                    ) : (
                      <XCircle className="mt-0.5 size-5 shrink-0 text-red-600" aria-hidden />
                    )}
                    <div className="flex-1">
                      <p className={result.success ? "font-medium text-emerald-700" : "font-medium text-red-700"}>
                        {result.message}
                      </p>
                      {result.item_url ? (
                        <a
                          href={result.item_url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="mt-1 flex items-center gap-1 text-[length:var(--text-sm)] text-primary hover:underline"
                        >
                          <ExternalLink className="size-3" aria-hidden />
                          查看商品
                        </a>
                      ) : null}
                    </div>
                  </div>
                </div>
              ) : null}
            </div>
          </div>
        )}
      </div>

      {/* 素材选择弹窗 */}
      {showPicker ? (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
          <div className="flex max-h-[80vh] w-full max-w-2xl flex-col rounded-xl bg-background shadow-2xl">
            <div className="flex items-center justify-between border-b border-border px-4 py-3">
              <h3 className="font-medium">从素材库导入</h3>
              <Button size="sm" variant="ghost" onClick={() => setShowPicker(false)}>
                关闭
              </Button>
            </div>
            <div className="flex-1 overflow-auto p-4">
              {materials.length === 0 ? (
                <div className="py-10 text-center text-muted-foreground">素材库为空</div>
              ) : (
                <div className="space-y-2">
                  {materials.map((material) => (
                    <button
                      key={material.id}
                      type="button"
                      onClick={() => applyMaterial(material)}
                      className="flex w-full items-center justify-between rounded-lg border border-border px-3 py-2 text-left transition-colors hover:bg-muted/40"
                    >
                      <div className="min-w-0">
                        <div className="truncate text-[length:var(--text-sm)] font-medium">
                          {material.title}
                        </div>
                        <div className="text-[length:var(--text-xs)] text-muted-foreground">
                          {material.category ?? "-"} · {material.condition} · ¥{material.price}
                        </div>
                      </div>
                    </button>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>
      ) : null}
    </PageScaffold>
  );
}
