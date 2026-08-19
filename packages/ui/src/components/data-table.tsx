/**
 * DataTable — shadcn Table + TanStack Table，可选接入 TanStack Query。
 *
 * Feature 负责 `queryFn`（IPC）；本组件不发起请求、不感知业务字段。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */

import {
  flexRender,
  getCoreRowModel,
  getSortedRowModel,
  useReactTable,
  type ColumnDef,
  type OnChangeFn,
  type PaginationState,
  type Row,
  type SortingState,
} from "@tanstack/react-table";
import type { UseQueryResult } from "@tanstack/react-query";

import { Button } from "./button";
import { Loading } from "./loading";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "./table";

/**
 * 服务端分页查询的标准载荷。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */
export type DataTablePage<TData> = {
  /** 当前页行。 */
  rows: TData[];
  /** 总行数（用于页码）。 */
  total: number;
};

/**
 * Query 可返回行数组，或 `{ rows, total }`。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */
export type DataTableQueryData<TData> = TData[] | DataTablePage<TData>;

/**
 * DataTable 列定义（从 TanStack Table 再导出，Feature 不必直连该库）。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */
export type { ColumnDef, PaginationState, SortingState };

/**
 * DataTable 属性。
 *
 * `query` 与 `data` 二选一：有 Query 时以 Query 为准。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */
export type DataTableProps<TData> = {
  /** 列定义。 */
  columns: ColumnDef<TData, unknown>[];
  /** 客户端数据；与 `query` 同时传入时忽略。 */
  data?: TData[];
  /** TanStack Query 结果。 */
  query?: Pick<
    UseQueryResult<DataTableQueryData<TData>>,
    "data" | "isPending" | "isError" | "error" | "refetch"
  >;
  /** 行 id；默认用索引。 */
  getRowId?: (originalRow: TData, index: number, parent?: Row<TData>) => string;
  /** 空态文案。 */
  emptyText?: string;
  /** 受控分页（传入即视为服务端分页）。 */
  pagination?: PaginationState;
  /** 分页变更。 */
  onPaginationChange?: OnChangeFn<PaginationState>;
  /** 受控排序。 */
  sorting?: SortingState;
  /** 排序变更。 */
  onSortingChange?: OnChangeFn<SortingState>;
};

function isPageResult<TData>(data: DataTableQueryData<TData>): data is DataTablePage<TData> {
  return !Array.isArray(data);
}

function resolveSource<TData>(
  data: TData[] | undefined,
  query: DataTableProps<TData>["query"],
): {
  rows: TData[];
  total: number | undefined;
  isPending: boolean;
  isError: boolean;
  errorMessage: string | null;
  refetch?: () => void;
} {
  if (query) {
    const payload = query.data;
    if (payload && isPageResult(payload)) {
      return {
        rows: payload.rows,
        total: payload.total,
        isPending: query.isPending,
        isError: query.isError,
        errorMessage: query.error instanceof Error ? query.error.message : query.error ? String(query.error) : null,
        refetch: query.refetch,
      };
    }
    const rows = Array.isArray(payload) ? payload : [];
    return {
      rows,
      total: undefined,
      isPending: query.isPending,
      isError: query.isError,
      errorMessage: query.error instanceof Error ? query.error.message : query.error ? String(query.error) : null,
      refetch: query.refetch,
    };
  }
  return {
    rows: data ?? [],
    total: undefined,
    isPending: false,
    isError: false,
    errorMessage: null,
  };
}

/**
 * 数据表格。
 *
 * 负责：
 * - 用 TanStack Table 渲染列与行
 * - 把 Query 的 pending / error / `{ rows, total }` 映射到 UI
 * - 服务端分页条（上一页 / 下一页）
 *
 * @author Xiaoman
 * @created 2026-08-19
 *
 * @param props - 见 {@link DataTableProps}
 * @returns 表格节点
 */
export function DataTable<TData>({
  columns,
  data,
  query,
  getRowId,
  emptyText = "暂无数据",
  pagination,
  onPaginationChange,
  sorting,
  onSortingChange,
}: DataTableProps<TData>) {
  const source = resolveSource(data, query);
  const manualPagination = pagination !== undefined;
  const enableSorting = sorting !== undefined || onSortingChange !== undefined;
  const pageCount =
    manualPagination && source.total !== undefined
      ? Math.max(1, Math.ceil(source.total / Math.max(1, pagination.pageSize)))
      : undefined;

  // React Compiler 会跳过本组件：useReactTable 返回的函数无法安全 memo。
  // eslint-disable-next-line react-hooks/incompatible-library -- TanStack Table 官方不兼容 Compiler memo
  const table = useReactTable({
    data: source.rows,
    columns,
    getRowId,
    getCoreRowModel: getCoreRowModel(),
    ...(enableSorting ? { getSortedRowModel: getSortedRowModel() } : {}),
    enableSorting,
    manualPagination,
    manualSorting: enableSorting && onSortingChange !== undefined,
    pageCount,
    state: {
      ...(pagination ? { pagination } : {}),
      ...(sorting ? { sorting } : {}),
    },
    onPaginationChange,
    onSortingChange,
  });

  if (source.isPending && source.rows.length === 0) {
    return <Loading size="lg" text="加载中..." className="py-16" />;
  }

  if (source.isError && source.rows.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center gap-3 py-16 text-center">
        <p className="text-(length:--text-sm) text-destructive">{source.errorMessage ?? "加载失败"}</p>
        {source.refetch ? (
          <Button size="sm" variant="outline" onClick={() => void source.refetch?.()}>
            重试
          </Button>
        ) : null}
      </div>
    );
  }

  if (source.rows.length === 0) {
    return (
      <div className="py-16 text-center text-(length:--text-sm) text-muted-foreground">{emptyText}</div>
    );
  }

  const pageIndex = pagination?.pageIndex ?? 0;
  const totalPages = pageCount ?? 1;
  const showPager = manualPagination && totalPages > 1;

  return (
    <div className="space-y-4">
      <Table>
        <TableHeader>
          {table.getHeaderGroups().map((headerGroup) => (
            <TableRow key={headerGroup.id}>
              {headerGroup.headers.map((header) => {
                const canSort = header.column.getCanSort();
                const content = header.isPlaceholder
                  ? null
                  : flexRender(header.column.columnDef.header, header.getContext());
                return (
                  <TableHead key={header.id} colSpan={header.colSpan}>
                    {canSort ? (
                      <button
                        type="button"
                        className="inline-flex items-center gap-1 font-medium"
                        onClick={header.column.getToggleSortingHandler()}
                      >
                        {content}
                      </button>
                    ) : (
                      content
                    )}
                  </TableHead>
                );
              })}
            </TableRow>
          ))}
        </TableHeader>
        <TableBody>
          {table.getRowModel().rows.map((row) => (
            <TableRow key={row.id}>
              {row.getVisibleCells().map((cell) => (
                <TableCell key={cell.id}>
                  {flexRender(cell.column.columnDef.cell, cell.getContext())}
                </TableCell>
              ))}
            </TableRow>
          ))}
        </TableBody>
      </Table>

      {showPager ? (
        <div className="flex items-center justify-end gap-2">
          <Button
            size="sm"
            variant="outline"
            disabled={pageIndex <= 0}
            onClick={() => table.previousPage()}
          >
            上一页
          </Button>
          <span className="text-(length:--text-sm) text-muted-foreground">
            {pageIndex + 1} / {totalPages}
          </span>
          <Button
            size="sm"
            variant="outline"
            disabled={pageIndex + 1 >= totalPages}
            onClick={() => table.nextPage()}
          >
            下一页
          </Button>
        </div>
      ) : null}
    </div>
  );
}
