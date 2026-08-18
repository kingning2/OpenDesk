/**
 * 闲鱼地址库页（迁移自原前端 `pages/product-publish/PublishAddresses.tsx` + Global/PersonalAddressTab）。
 *
 * 按原前端核心交互重写：随机地址库（全局池）与个人地址库两个 Tab，
 * 各自支持分页列表 + 关键词筛选 + 新建/编辑 + 单删/批量删除。
 * 数据访问走 Tauri IPC（`@desk/platform/ipc/address`），复用 crates/app AddressService。
 *
 * 说明：原前端个人库的 Excel 导入/导出依赖后端文件处理，桌面端不迁移；
 * 全局池管理员维护语义在桌面单用户下为当前用户维护。
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
import { MapPin, Pencil, Plus, RefreshCw, Trash2 } from "@desk/ui/icons";
import {
  addressBatchDelete,
  addressCreate,
  addressDelete,
  addressList,
  addressUpdate,
  type AddressType,
  type PublishAddress,
} from "@desk/platform/ipc/address";

const OWNER_ID = 1; // 桌面单用户；多用户时由登录态注入
const PAGE_SIZE_OPTIONS = [10, 20, 50, 100];

type AddressTab = "global" | "personal";

/** 地址表单。 */
interface AddressForm {
  address: string;
  name: string;
  searchKeyword: string;
  expectedText: string;
  weight: string;
  remark: string;
}

const EMPTY_FORM: AddressForm = {
  address: "",
  name: "",
  searchKeyword: "",
  expectedText: "",
  weight: "1",
  remark: "",
};

function toForm(address: PublishAddress): AddressForm {
  return {
    address: address.address,
    name: address.name,
    searchKeyword: address.search_keyword,
    expectedText: address.expected_text ?? "",
    weight: String(address.weight),
    remark: address.remark ?? "",
  };
}

/** 单个 Tab 的列表区（全局/个人共用交互）。 */
function AddressTable({
  tab,
  keyword,
  onKeywordChange,
}: {
  tab: AddressType;
  keyword: string;
  onKeywordChange: (value: string) => void;
}) {
  const [addresses, setAddresses] = useState<PublishAddress[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [loading, setLoading] = useState(true);
  const [tableLoading, setTableLoading] = useState(false);
  const [selectedIds, setSelectedIds] = useState<number[]>([]);
  const [formOpen, setFormOpen] = useState(false);
  const [editing, setEditing] = useState<PublishAddress | null>(null);
  const [form, setForm] = useState<AddressForm>(EMPTY_FORM);
  const [saving, setSaving] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<PublishAddress | null>(null);
  const [batchDeleteConfirm, setBatchDeleteConfirm] = useState(false);
  const [deleting, setDeleting] = useState(false);

  async function load(nextPage = page, nextPageSize = pageSize) {
    setTableLoading(true);
    try {
      const [list, count] = await addressList({
        page: nextPage,
        page_size: nextPageSize,
        keyword: keyword.trim() || undefined,
        address_type: tab,
      });
      setAddresses(list);
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
    void addressList({ page: 1, page_size: 20, address_type: tab })
      .then(([list, count]) => {
        if (cancelled) return;
        setAddresses(list);
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
  }, [tab]);

  function openCreate() {
    setEditing(null);
    setForm(EMPTY_FORM);
    setFormOpen(true);
  }

  function openEdit(address: PublishAddress) {
    setEditing(address);
    setForm(toForm(address));
    setFormOpen(true);
  }

  async function handleSubmit() {
    if (!form.address.trim()) {
      toast.error("地址不能为空");
      return;
    }
    setSaving(true);
    try {
      const payload = {
        address: form.address.trim(),
        name: form.name.trim(),
        search_keyword: form.searchKeyword.trim(),
        expected_text: form.expectedText.trim() || null,
        weight: Math.max(1, Number(form.weight) || 1),
        sort_order: 0,
        is_enabled: true,
        use_count: 0,
        remark: form.remark.trim() || null,
        created_at: null,
        updated_at: null,
      };
      if (editing) {
        await addressUpdate({ ...payload, id: editing.id, owner_id: OWNER_ID, address_type: tab });
        toast.success("地址已更新");
      } else {
        await addressCreate({ ...payload, address_type: tab });
        toast.success("地址已添加");
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
      await addressDelete(deleteTarget.id);
      toast.success("地址已删除");
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
      const count = await addressBatchDelete(selectedIds);
      toast.success(`成功删除 ${count} 条地址`);
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
    if (addresses.length === 0) return;
    const pageIds = addresses.map((item) => item.id);
    const allSelected = pageIds.every((id) => selectedIds.includes(id));
    setSelectedIds((prev) =>
      allSelected ? prev.filter((id) => !pageIds.includes(id)) : [...new Set([...prev, ...pageIds])],
    );
  };

  const allCurrentSelected = addresses.length > 0 && addresses.every((a) => selectedIds.includes(a.id));

  return (
    <div className="space-y-4">
      {/* 工具栏 */}
      <div className="flex items-center justify-between gap-3">
        <div className="flex gap-2">
          <Input
            value={keyword}
            onChange={(event) => onKeywordChange(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") void load(1);
            }}
            placeholder="搜索地址..."
            className="w-48"
          />
          <Button size="sm" onClick={() => void load(1)}>查询</Button>
          {keyword.trim() ? (
            <Button
              size="sm"
              variant="outline"
              onClick={() => {
                onKeywordChange("");
                void load(1);
              }}
            >
              重置
            </Button>
          ) : null}
        </div>
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
          <Button size="sm" onClick={openCreate}>
            <Plus className="size-4" aria-hidden />
            新建地址
          </Button>
        </div>
      </div>

      {/* 地址列表 */}
      {loading ? (
        <Loading size="lg" text="加载中..." className="py-16" />
      ) : addresses.length === 0 ? (
        <div className="py-16 text-center text-muted-foreground">
          <MapPin className="mx-auto mb-2 size-10 opacity-40" aria-hidden />
          暂无地址
        </div>
      ) : (
        <div className="overflow-hidden rounded-xl border border-border">
          <div className="flex items-center justify-between border-b border-border bg-muted/30 px-4 py-2">
            <span className="text-[length:var(--text-sm)] font-medium">
              {tab === "global" ? "随机地址池" : "个人地址库"}
            </span>
            <span className="rounded-full bg-primary/15 px-2 py-0.5 text-[length:var(--text-xs)] text-primary">
              共 {total} 条
            </span>
          </div>
          <div className="max-h-[55vh] overflow-auto">
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
                  <th className="px-3 py-2 text-left font-medium">地址</th>
                  <th className="px-3 py-2 text-left font-medium">名称</th>
                  <th className="px-3 py-2 text-left font-medium">权重</th>
                  <th className="px-3 py-2 text-left font-medium">使用次数</th>
                  <th className="px-3 py-2 text-left font-medium">创建时间</th>
                  <th className="px-3 py-2 text-right font-medium">操作</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border">
                {tableLoading ? (
                  <tr>
                    <td colSpan={7} className="py-12 text-center text-muted-foreground">加载中...</td>
                  </tr>
                ) : (
                  addresses.map((address) => (
                    <tr
                      key={address.id}
                      className={selectedIds.includes(address.id) ? "bg-primary/5" : "hover:bg-muted/30"}
                    >
                      <td className="px-3 py-2">
                        <input
                          type="checkbox"
                          checked={selectedIds.includes(address.id)}
                          onChange={() => toggleSelect(address.id)}
                          className="size-4 accent-primary"
                          aria-label={`选择 ${address.address}`}
                        />
                      </td>
                      <td className="max-w-56 px-3 py-2">
                        <span className="block truncate" title={address.address}>
                          {address.address}
                        </span>
                      </td>
                      <td className="px-3 py-2 text-muted-foreground">{address.name || "-"}</td>
                      <td className="px-3 py-2">{address.weight}</td>
                      <td className="px-3 py-2 text-muted-foreground">{address.use_count}</td>
                      <td className="whitespace-nowrap px-3 py-2 text-muted-foreground">
                        {address.created_at ?? "-"}
                      </td>
                      <td className="px-3 py-2 text-right">
                        <div className="flex items-center justify-end gap-1">
                          <Button size="sm" variant="ghost" onClick={() => openEdit(address)}>
                            <Pencil className="size-3.5" aria-hidden />
                          </Button>
                          <Button
                            size="sm"
                            variant="ghost"
                            className="text-destructive"
                            onClick={() => setDeleteTarget(address)}
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

      {/* 新建/编辑弹窗 */}
      <ConfirmModal
        isOpen={formOpen}
        title={editing ? "编辑地址" : "新建地址"}
        message={
          <div className="space-y-3 text-left">
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">地址 *</span>
              <Textarea
                value={form.address}
                onChange={(event) => setForm((current) => ({ ...current, address: event.target.value }))}
                rows={2}
                placeholder="如：北京市朝阳区望京街道..."
              />
            </label>
            <label className="block space-y-1">
              <span className="text-[length:var(--text-sm)] text-muted-foreground">名称</span>
              <Input
                value={form.name}
                onChange={(event) => setForm((current) => ({ ...current, name: event.target.value }))}
                placeholder="地址名称"
              />
            </label>
            <div className="grid grid-cols-2 gap-3">
              <label className="block space-y-1">
                <span className="text-[length:var(--text-sm)] text-muted-foreground">搜索词</span>
                <Input
                  value={form.searchKeyword}
                  onChange={(event) => setForm((current) => ({ ...current, searchKeyword: event.target.value }))}
                  placeholder="搜索关键词"
                />
              </label>
              <label className="block space-y-1">
                <span className="text-[length:var(--text-sm)] text-muted-foreground">权重</span>
                <Input
                  type="number"
                  min={1}
                  value={form.weight}
                  onChange={(event) => setForm((current) => ({ ...current, weight: event.target.value }))}
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
        message={`确认删除该地址？`}
        confirmText="删除"
        loading={deleting}
        onConfirm={() => void handleDelete()}
        onCancel={() => setDeleteTarget(null)}
      />

      {/* 批量删除确认 */}
      <ConfirmModal
        isOpen={batchDeleteConfirm}
        type="danger"
        title="确认批量删除"
        message={`确认删除选中的 ${selectedIds.length} 条地址？`}
        confirmText={`删除 ${selectedIds.length} 条`}
        loading={deleting}
        onConfirm={() => void handleBatchDelete()}
        onCancel={() => setBatchDeleteConfirm(false)}
      />
    </div>
  );
}

/**
 * 闲鱼地址库页。
 *
 * @author agent
 * @created 2026-08-13
 */
export function XianyuPublishAddressesPage() {
  const [activeTab, setActiveTab] = useState<AddressTab>("global");
  const [keyword, setKeyword] = useState("");

  return (
    <PageScaffold subtitle="闲鱼地址库 — 随机地址池与个人地址，发布时回退使用">
      {/* Tab 切换 */}
      <div className="mb-4 flex border-b border-border">
        <button
          type="button"
          onClick={() => {
            setActiveTab("global");
            setKeyword("");
          }}
          className={`border-b-2 px-4 py-2 text-[length:var(--text-sm)] font-medium transition-colors ${
            activeTab === "global"
              ? "border-primary text-primary"
              : "border-transparent text-muted-foreground hover:text-foreground"
          }`}
        >
          随机地址库
        </button>
        <button
          type="button"
          onClick={() => {
            setActiveTab("personal");
            setKeyword("");
          }}
          className={`border-b-2 px-4 py-2 text-[length:var(--text-sm)] font-medium transition-colors ${
            activeTab === "personal"
              ? "border-primary text-primary"
              : "border-transparent text-muted-foreground hover:text-foreground"
          }`}
        >
          个人地址库
        </button>
      </div>

      <AddressTable
        key={activeTab}
        tab={activeTab}
        keyword={keyword}
        onKeywordChange={setKeyword}
      />
    </PageScaffold>
  );
}
