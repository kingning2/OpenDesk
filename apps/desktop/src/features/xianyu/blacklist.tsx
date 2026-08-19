/**
 * 闲鱼黑名单管理页（迁移自原前端 `pages/blacklist/PersonalBlacklist.tsx`）。
 *
 * 列表：DataTable + TanStack Query；新增：FormInput / FormTextarea + Zod。
 * 数据访问走 Tauri IPC（`@desk/platform/ipc/blacklist`）。
 */

import { useState } from "react";
import {
  Button,
  ConfirmModal,
  DataTable,
  Dialog,
  DialogContent,
  DialogFooter,
  Form,
  FormInput,
  FormTextarea,
  Input,
  PageScaffold,
  toast,
  useMutation,
  useQuery,
  useQueryClient,
  z,
  type ColumnDef,
  type PaginationState,
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
const BLACKLIST_QUERY_KEY = ["xianyu", "blacklist"] as const;

const createSchema = z.object({
  buyerIds: z.string().trim().min(1, "买家 ID 不能为空"),
  reason: z.string().optional(),
});

/**
 * 闲鱼黑名单管理页。
 *
 * @author Xiaoman
 * @created 2026-08-19
 *
 * @returns 黑名单页节点
 */
export function XianyuBlacklistPage() {
  const queryClient = useQueryClient();
  const [searchInput, setSearchInput] = useState("");
  const [search, setSearch] = useState("");
  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: 0,
    pageSize: PAGE_SIZE,
  });
  const [showForm, setShowForm] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<PersonalBlacklistItem | null>(null);

  const page = pagination.pageIndex + 1;
  const listQuery = useQuery({
    queryKey: [...BLACKLIST_QUERY_KEY, OWNER_ID, page, search],
    queryFn: async () => {
      const [rows, total] = await blacklistPersonalList({
        owner_id: OWNER_ID,
        page,
        page_size: pagination.pageSize,
        buyer_id: search.trim() || undefined,
      });
      return { rows, total };
    },
  });

  const createMutation = useMutation({
    mutationFn: (values: z.infer<typeof createSchema>) =>
      blacklistPersonalCreate({
        owner_id: OWNER_ID,
        buyer_ids: values.buyerIds,
        reason: values.reason?.trim() || undefined,
      }),
    onSuccess: async () => {
      toast.success("已加入黑名单");
      setShowForm(false);
      await queryClient.invalidateQueries({ queryKey: BLACKLIST_QUERY_KEY });
    },
    onError: (error: unknown) => {
      toast.error(error instanceof Error ? error.message : String(error));
    },
  });

  const toggleMutation = useMutation({
    mutationFn: (item: PersonalBlacklistItem) =>
      blacklistSetEnabled(OWNER_ID, item.id, !item.is_enabled),
    onSuccess: async (_void, item) => {
      toast.success(item.is_enabled ? "已停用" : "已启用");
      await queryClient.invalidateQueries({ queryKey: BLACKLIST_QUERY_KEY });
    },
    onError: (error: unknown) => {
      toast.error(error instanceof Error ? error.message : String(error));
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (item: PersonalBlacklistItem) => blacklistDelete(OWNER_ID, item.id),
    onSuccess: async () => {
      toast.success("已移除黑名单");
      setDeleteTarget(null);
      await queryClient.invalidateQueries({ queryKey: BLACKLIST_QUERY_KEY });
    },
    onError: (error: unknown) => {
      toast.error(error instanceof Error ? error.message : String(error));
    },
  });

  function applySearch() {
    setSearch(searchInput.trim());
    setPagination((current) => ({ ...current, pageIndex: 0 }));
  }

  const columns: ColumnDef<PersonalBlacklistItem>[] = [
    {
      accessorKey: "buyer_id",
      header: "买家 ID",
      cell: ({ row }) => <span className="font-mono">{row.original.buyer_id}</span>,
    },
    {
      id: "level",
      header: "级别",
      cell: ({ row }) => (
        <span className="rounded-full bg-muted px-2 py-0.5 text-(length:--text-xs)">
          {blacklistLevel(row.original)}
        </span>
      ),
    },
    {
      accessorKey: "reason",
      header: "原因",
      cell: ({ row }) => (
        <span className="text-muted-foreground">{row.original.reason || "—"}</span>
      ),
    },
    {
      accessorKey: "is_enabled",
      header: "状态",
      cell: ({ row }) =>
        row.original.is_enabled ? (
          <span className="rounded-full bg-red-500/15 px-2 py-0.5 text-(length:--text-xs) text-red-500">
            生效中
          </span>
        ) : (
          <span className="rounded-full bg-muted px-2 py-0.5 text-(length:--text-xs) text-muted-foreground">
            已停用
          </span>
        ),
    },
    {
      id: "actions",
      header: () => <span className="flex justify-end">操作</span>,
      cell: ({ row }) => {
        const item = row.original;
        return (
          <div className="flex items-center justify-end gap-1">
            <Button
              size="sm"
              variant="outline"
              disabled={toggleMutation.isPending}
              onClick={() => toggleMutation.mutate(item)}
            >
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
        );
      },
    },
  ];

  return (
    <PageScaffold subtitle="闲鱼黑名单管理 — 禁止发货买家">
      <div className="space-y-4">
        <div className="flex items-center justify-between gap-3">
          <div className="flex items-center gap-3">
            <Input
              placeholder="搜索买家 ID"
              value={searchInput}
              onChange={(event) => setSearchInput(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter") applySearch();
              }}
              className="w-56"
            />
            <Button variant="outline" size="sm" onClick={applySearch}>
              搜索
            </Button>
          </div>
          <Button onClick={() => setShowForm(true)}>
            <Plus className="size-4" aria-hidden />
            加入黑名单
          </Button>
        </div>

        <DataTable
          columns={columns}
          query={listQuery}
          getRowId={(row) => String(row.id)}
          emptyText="暂无黑名单记录"
          pagination={pagination}
          onPaginationChange={setPagination}
        />
      </div>

      <Dialog open={showForm} onOpenChange={setShowForm}>
        <DialogContent title="加入黑名单">
          <Form
            key={String(showForm)}
            schema={createSchema}
            defaultValues={{ buyerIds: "", reason: "" }}
            onSubmit={async (values) => {
              try {
                await createMutation.mutateAsync(values);
              } catch {
                // onError 已 toast；吞掉避免 RHF 未处理的 Promise 拒绝
              }
            }}
          >
            {({ formState }) => (
              <>
                <FormTextarea
                  name="buyerIds"
                  label="买家 ID（支持多行批量）"
                  placeholder={"buyer-001\nbuyer-002"}
                  rows={3}
                />
                <FormInput name="reason" label="原因" placeholder="如：恶意退款买家" />
                <DialogFooter>
                  <Button type="button" variant="secondary" onClick={() => setShowForm(false)}>
                    取消
                  </Button>
                  <Button type="submit" disabled={formState.isSubmitting}>
                    {formState.isSubmitting ? "添加中…" : "添加"}
                  </Button>
                </DialogFooter>
              </>
            )}
          </Form>
        </DialogContent>
      </Dialog>

      <ConfirmModal
        isOpen={deleteTarget !== null}
        type="danger"
        title="移除黑名单"
        message={`确认将买家「${deleteTarget?.buyer_id ?? ""}」移出黑名单？`}
        confirmText="移除"
        loading={deleteMutation.isPending}
        onConfirm={() => {
          if (deleteTarget) deleteMutation.mutate(deleteTarget);
        }}
        onCancel={() => setDeleteTarget(null)}
      />
    </PageScaffold>
  );
}
