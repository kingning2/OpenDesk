/**
 * 闲鱼商品素材库页（迁移自原前端 `pages/product-publish/ProductMaterials.tsx`）。
 *
 * 按原前端核心交互重写：素材分页列表 + 标题/分类/成色/平台分类筛选 +
 * 新建/编辑 + 单删 + 批量删除。
 * 数据访问走 Tauri IPC（`@desk/platform/ipc/publish-material`），复用 crates/app PublishMaterialService。
 *
 * 说明：原弹窗的平台分类推荐 / 规格 / SKU 编辑器依赖外部服务，桌面端以核心字段表单替代。
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
import { Image as ImageIcon, Pencil, Plus, RefreshCw, Trash2 } from "@desk/ui/icons";
import {
  publishMaterialBatchDelete,
  publishMaterialCreate,
  publishMaterialDelete,
  publishMaterialList,
  publishMaterialUpdate,
  type PublishMaterial,
} from "@desk/platform/ipc/publish-material";

const OWNER_ID = 1; // 桌面单用户；多用户时由登录态注入
const PAGE_SIZE_OPTIONS = [10, 20, 50, 100];
const CONDITIONS = ["全新", "99新", "95新", "9成新", "8成新", "7成新以下"];

/** 筛选条件。 */
interface Filters {
  title: string;
  category: string;
  condition: string;
  platformCategoryId: string;
}

const EMPTY_FILTERS: Filters = {
  title: "",
  category: "",
  condition: "",
  platformCategoryId: "",
};

/** 素材表单（核心字段）。 */
interface MaterialForm {
  title: string;
  description: string;
  price: string;
  original_price: string;
  category: string;
  condition: string;
  images: string;
  quantity: string;
  delivery_method: string;
  shipping_method: string;
  postage: string;
  brand: string;
  remark: string;
}

const EMPTY_FORM: MaterialForm = {
  title: "",
  description: "",
  price: "",
  original_price: "",
  category: "",
  condition: "全新",
  images: "",
  quantity: "1",
  delivery_method: "express",
  shipping_method: "free",
  postage: "0",
  brand: "",
  remark: "",
};

function toForm(material: PublishMaterial): MaterialForm {
  return {
    title: material.title,
    description: material.description,
    price: String(material.price),
    original_price: material.original_price == null ? "" : String(material.original_price),
    category: material.category ?? "",
    condition: material.condition,
    images: material.images,
    quantity: String(material.quantity),
    delivery_method: material.delivery_method,
    shipping_method: material.shipping_method,
    postage: String(material.postage),
    brand: material.brand ?? "",
    remark: material.remark ?? "",
  };
}

/**
 * 闲鱼商品素材库页。
 *
 * @author agent
 * @created 2026-08-13
 */
export function XianyuProductMaterialsPage() {
  const [materials, setMaterials] = useState<PublishMaterial[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [loading, setLoading] = useState(true);
  const [tableLoading, setTableLoading] = useState(false);
  const [filters, setFilters] = useState<Filters>(EMPTY_FILTERS);
  const [selectedIds, setSelectedIds] = useState<number[]>([]);
  const [formOpen, setFormOpen] = useState(false);
  const [editing, setEditing] = useState<PublishMaterial | null>(null);
  const [form, setForm] = useState<MaterialForm>(EMPTY_FORM);
  const [saving, setSaving] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<PublishMaterial | null>(null);
  const [batchDeleteConfirm, setBatchDeleteConfirm] = useState(false);
  const [deleting, setDeleting] = useState(false);

  async function load(nextPage = page, nextPageSize = pageSize) {
    setTableLoading(true);
    try {
      const [list, count] = await publishMaterialList({
        page: nextPage,
        page_size: nextPageSize,
        keyword: filters.title.trim() || undefined,
        category: filters.category.trim() || undefined,
        condition: filters.condition || undefined,
        platform_category_id: filters.platformCategoryId.trim() || undefined,
      });
      setMaterials(list);
      setTotal(count);
      setPage(nextPage);
      setPageSize(nextPageSize);
      const currentIds = new Set(list.map((item) => item.id));
      setSelectedIds((prev) => prev.filter((id) => currentIds.has(id)));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setLoading(false);
      setTableLoading(false);
    }
  }

  useEffect(() => {
    let cancelled = false;
    void publishMaterialList({ page: 1, page_size: 20 })
      .then(([list, count]) => {
        if (cancelled) return;
        setMaterials(list);
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

  const patchFilters = (patch: Partial<Filters>) =>
    setFilters((current) => ({ ...current, ...patch }));

  const hasActiveFilters =
    filters.title.trim() !== "" ||
    filters.category.trim() !== "" ||
    filters.condition !== "" ||
    filters.platformCategoryId.trim() !== "";

  function handleFilter() {
    setPage(1);
    setSelectedIds([]);
    void load(1, pageSize);
  }

  function handleResetFilter() {
    setFilters(EMPTY_FILTERS);
    setPage(1);
    setSelectedIds([]);
    void load(1, pageSize);
  }

  function openCreate() {
    setEditing(null);
    setForm(EMPTY_FORM);
    setFormOpen(true);
  }

  function openEdit(material: PublishMaterial) {
    setEditing(material);
    setForm(toForm(material));
    setFormOpen(true);
  }

  async function handleSubmit() {
    if (!form.title.trim()) {
      toast.error("素材标题不能为空");
      return;
    }
    const price = Number(form.price);
    if (!Number.isFinite(price) || price < 0) {
      toast.error("素材价格必须是非负数字");
      return;
    }
    const originalPrice = form.original_price.trim() ? Number(form.original_price) : null;
    setSaving(true);
    try {
      const payload = {
        title: form.title.trim(),
        description: form.description,
        price,
        original_price: originalPrice,
        category: form.category.trim() || null,
        platform_category_id: null,
        platform_category_name: null,
        images: form.images.trim(),
        condition: form.condition,
        quantity: Math.max(1, Number(form.quantity) || 1),
        delivery_method: form.delivery_method,
        shipping_method: form.shipping_method,
        postage: Number(form.postage) || 0,
        brand: form.brand.trim() || null,
        remark: form.remark.trim() || null,
        created_at: null,
        updated_at: null,
      };
      if (editing) {
        await publishMaterialUpdate({ ...payload, id: editing.id, owner_id: OWNER_ID });
        toast.success("素材已更新");
      } else {
        await publishMaterialCreate(payload);
        toast.success("素材已创建");
      }
      setFormOpen(false);
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
      await publishMaterialDelete(deleteTarget.id);
      toast.success("素材已移出素材库");
      setDeleteTarget(null);
      setSelectedIds((prev) => prev.filter((id) => id !== deleteTarget.id));
      await load();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setDeleting(false);
    }
  }

  async function handleBatchDelete() {
    setDeleting(true);
    try {
      const count = await publishMaterialBatchDelete(selectedIds);
      toast.success(`已移出 ${count} 条素材`);
      setBatchDeleteConfirm(false);
      setSelectedIds([]);
      await load();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error));
    } finally {
      setDeleting(false);
    }
  }

  const toggleSelect = (id: number) =>
    setSelectedIds((prev) =>
      prev.includes(id) ? prev.filter((item) => item !== id) : [...prev, id],
    );

  const toggleSelectAll = () => {
    if (materials.length === 0) return;
    const pageIds = materials.map((item) => item.id);
    const allSelected = pageIds.every((id) => selectedIds.includes(id));
    setSelectedIds((prev) =>
      allSelected ? prev.filter((id) => !pageIds.includes(id)) : [...new Set([...prev, ...pageIds])],
    );
  };

  const allCurrentSelected = materials.length > 0 && materials.every((m) => selectedIds.includes(m.id));

  function imageCount(material: PublishMaterial): number {
    if (!material.images.trim()) return 0;
    try {
      const parsed = JSON.parse(material.images) as unknown;
      return Array.isArray(parsed) ? parsed.length : 0;
    } catch {
      return 0;
    }
  }

  return (
    <PageScaffold subtitle="闲鱼商品素材库 — 管理发布素材，供单品发布和批量发布引用">
      <div className="space-y-4">
        {/* 工具栏 */}
        <div className="flex items-center justify-between gap-3">
          <div className="flex gap-2">
            {selectedIds.length > 0 ? (
              <Button
                size="sm"
                variant="outline"
                className="text-destructive"
                onClick={() => setBatchDeleteConfirm(true)}
              >
                <Trash2 className="size-4" aria-hidden />
                批量删除 ({selectedIds.length})
              </Button>
            ) : null}
            <Button size="sm" variant="outline" onClick={() => void load()} disabled={tableLoading}>
              <RefreshCw className="size-4" aria-hidden />
              刷新
            </Button>
          </div>
          <Button onClick={openCreate}>
            <Plus className="size-4" aria-hidden />
            新建素材
          </Button>
        </div>

        {/* 筛选栏 */}
        <div className="rounded-xl border border-border bg-shell p-4">
          <div className="flex flex-wrap items-end gap-3">
            <label className="block space-y-1">
              <span className="text-[length:var(--text-xs)] text-muted-foreground">搜索标题</span>
              <Input
                value={filters.title}
                onChange={(event) => patchFilters({ title: event.target.value })}
                onKeyDown={(event) => {
                  if (event.key === "Enter") handleFilter();
                }}
                placeholder="搜索标题..."
                className="w-48"
              />
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-xs)] text-muted-foreground">本地分类</span>
              <Input
                value={filters.category}
                onChange={(event) => patchFilters({ category: event.target.value })}
                placeholder="本地分类..."
                className="w-40"
              />
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-xs)] text-muted-foreground">平台分类ID</span>
              <Input
                value={filters.platformCategoryId}
                onChange={(event) => patchFilters({ platformCategoryId: event.target.value })}
                placeholder="平台分类ID..."
                className="w-40"
              />
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-xs)] text-muted-foreground">成色</span>
              <Select value={filters.condition} onValueChange={(value) => patchFilters({ condition: value })}>
                <SelectTrigger className="w-28" aria-label="成色">
                  <SelectValue placeholder="全部成色" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="">全部成色</SelectItem>
                  {CONDITIONS.map((condition) => (
                    <SelectItem key={condition} value={condition}>
                      {condition}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </label>
            <Button size="sm" onClick={handleFilter}>查询</Button>
            {hasActiveFilters ? (
              <Button size="sm" variant="outline" onClick={handleResetFilter}>
                重置
              </Button>
            ) : null}
          </div>
        </div>

        {/* 素材列表 */}
        {loading ? (
          <Loading size="lg" text="加载中..." className="py-16" />
        ) : materials.length === 0 ? (
          <div className="py-16 text-center text-muted-foreground">
            <ImageIcon className="mx-auto mb-2 size-10 opacity-40" aria-hidden />
            暂无素材，点击「新建素材」添加
          </div>
        ) : (
          <div className="overflow-hidden rounded-xl border border-border">
            <div className="flex items-center justify-between border-b border-border bg-muted/30 px-4 py-2">
              <span className="text-[length:var(--text-sm)] font-medium">素材列表</span>
              <span className="rounded-full bg-primary/15 px-2 py-0.5 text-[length:var(--text-xs)] text-primary">
                共 {total} 条
              </span>
            </div>
            <div className="max-h-[60vh] overflow-auto">
              <table className="w-full text-[length:var(--text-xs)]">
                <thead className="sticky top-0 bg-muted/80 text-muted-foreground">
                  <tr>
                    <th className="w-10 px-3 py-2">
                      <input
                        type="checkbox"
                        checked={allCurrentSelected}
                        onChange={toggleSelectAll}
                        className="size-4 accent-primary"
                        aria-label="全选当前页"
                      />
                    </th>
                    <th className="px-3 py-2 text-left font-medium">标题</th>
                    <th className="px-3 py-2 text-left font-medium">价格</th>
                    <th className="px-3 py-2 text-left font-medium">分类</th>
                    <th className="px-3 py-2 text-left font-medium">成色</th>
                    <th className="px-3 py-2 text-left font-medium">媒体</th>
                    <th className="px-3 py-2 text-left font-medium">创建时间</th>
                    <th className="px-3 py-2 text-right font-medium">操作</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border">
                  {tableLoading ? (
                    <tr>
                      <td colSpan={8} className="py-12 text-center text-muted-foreground">加载中...</td>
                    </tr>
                  ) : (
                    materials.map((material) => (
                      <tr
                        key={material.id}
                        className={selectedIds.includes(material.id) ? "bg-primary/5" : "hover:bg-muted/30"}
                      >
                        <td className="px-3 py-2">
                          <input
                            type="checkbox"
                            checked={selectedIds.includes(material.id)}
                            onChange={() => toggleSelect(material.id)}
                            className="size-4 accent-primary"
                            aria-label={`选择 ${material.title}`}
                          />
                        </td>
                        <td className="max-w-48 px-3 py-2">
                          <span className="block truncate font-medium" title={material.title}>
                            {material.title}
                          </span>
                        </td>
                        <td className="px-3 py-2 font-medium text-amber-600">
                          {material.price}
                          {material.original_price ? (
                            <span className="ml-1 text-[length:var(--text-xs)] text-muted-foreground line-through">
                              {material.original_price}
                            </span>
                          ) : null}
                        </td>
                        <td className="px-3 py-2 text-muted-foreground">{material.category ?? "-"}</td>
                        <td className="px-3 py-2">
                          <span className="rounded-full bg-muted px-2 py-0.5 text-[length:var(--text-xs)]">
                            {material.condition}
                          </span>
                        </td>
                        <td className="px-3 py-2">
                          <span className="rounded-full bg-primary/10 px-2 py-0.5 text-[length:var(--text-xs)] text-primary">
                            {imageCount(material)} 图
                          </span>
                        </td>
                        <td className="whitespace-nowrap px-3 py-2 text-muted-foreground">
                          {material.created_at ?? "-"}
                        </td>
                        <td className="px-3 py-2 text-right">
                          <div className="flex items-center justify-end gap-1">
                            <Button size="sm" variant="ghost" onClick={() => openEdit(material)}>
                              <Pencil className="size-3.5" aria-hidden />
                            </Button>
                            <Button
                              size="sm"
                              variant="ghost"
                              className="text-destructive"
                              onClick={() => setDeleteTarget(material)}
                            >
                              <Trash2 className="size-3.5" aria-hidden />
                            </Button>
                          </div>
                        </td>
                      </tr>
                    ))
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

      {/* 新建/编辑弹窗 */}
      <ConfirmModal
        isOpen={formOpen}
        title={editing ? "编辑素材" : "新建素材"}
        message={
          <div className="space-y-3 text-left">
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">标题 *</span>
              <Input
                value={form.title}
                onChange={(event) => setForm((current) => ({ ...current, title: event.target.value }))}
                placeholder="素材标题"
              />
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">描述</span>
              <Textarea
                value={form.description}
                onChange={(event) => setForm((current) => ({ ...current, description: event.target.value }))}
                rows={2}
              />
            </label>
            <div className="grid grid-cols-2 gap-3">
              <label className="block space-y-1">
                <span className="text-[length:var(--text-sm)] text-muted-foreground">价格（元）*</span>
                <Input
                  type="number"
                  min={0}
                  step="0.01"
                  value={form.price}
                  onChange={(event) => setForm((current) => ({ ...current, price: event.target.value }))}
                />
              </label>
              <label className="block space-y-1">
                <span className="text-[length:var(--text-sm)] text-muted-foreground">原价（元）</span>
                <Input
                  type="number"
                  min={0}
                  step="0.01"
                  value={form.original_price}
                  onChange={(event) => setForm((current) => ({ ...current, original_price: event.target.value }))}
                />
              </label>
            </div>
            <div className="grid grid-cols-2 gap-3">
              <label className="block space-y-1">
                <span className="text-[length:var(--text-sm)] text-muted-foreground">本地分类</span>
                <Input
                  value={form.category}
                  onChange={(event) => setForm((current) => ({ ...current, category: event.target.value }))}
                  placeholder="如：数码"
                />
              </label>
              <label className="block space-y-1">
                <span className="text-[length:var(--text-sm)] text-muted-foreground">成色</span>
                <Select
                  value={form.condition}
                  onValueChange={(value) => setForm((current) => ({ ...current, condition: value }))}
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
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">
                图片 URL（JSON 数组，如 ["https://..."]）
              </span>
              <Textarea
                value={form.images}
                onChange={(event) => setForm((current) => ({ ...current, images: event.target.value }))}
                rows={2}
                className="font-mono text-[length:var(--text-xs)]"
                placeholder='["https://example.com/a.jpg"]'
              />
            </label>
            <div className="grid grid-cols-2 gap-3">
              <label className="block space-y-1">
                <span className="text-[length:var(--text-sm)] text-muted-foreground">数量</span>
                <Input
                  type="number"
                  min={1}
                  value={form.quantity}
                  onChange={(event) => setForm((current) => ({ ...current, quantity: event.target.value }))}
                />
              </label>
              <label className="block space-y-1">
                <span className="text-[length:var(--text-sm)] text-muted-foreground">邮费（元）</span>
                <Input
                  type="number"
                  min={0}
                  step="0.01"
                  value={form.postage}
                  onChange={(event) => setForm((current) => ({ ...current, postage: event.target.value }))}
                />
              </label>
            </div>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">备注</span>
              <Input
                value={form.remark}
                onChange={(event) => setForm((current) => ({ ...current, remark: event.target.value }))}
                placeholder="备注信息"
              />
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
        title="确认删除"
        message={`确认将素材「${deleteTarget?.title ?? ""}」移出素材库吗？历史发布日志不会受影响。`}
        confirmText="移出素材库"
        loading={deleting}
        onConfirm={() => void handleDelete()}
        onCancel={() => setDeleteTarget(null)}
      />

      {/* 批量删除确认 */}
      <ConfirmModal
        isOpen={batchDeleteConfirm}
        type="danger"
        title="确认批量移出"
        message={`确认将选中的 ${selectedIds.length} 条素材移出素材库吗？历史发布日志不会受影响。`}
        confirmText={`移出 ${selectedIds.length} 条`}
        loading={deleting}
        onConfirm={() => void handleBatchDelete()}
        onCancel={() => setBatchDeleteConfirm(false)}
      />
    </PageScaffold>
  );
}
