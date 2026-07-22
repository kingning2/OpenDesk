/**
 * TanStack Table + shadcn/ui table renderer.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */

import {
  flexRender,
  getCoreRowModel,
  useReactTable,
  type ColumnDef,
  type Table as TanstackTable,
} from "@tanstack/react-table";

import { cn } from "../lib/cn";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "./table";

declare module "@tanstack/react-table" {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  interface ColumnMeta<TData, TValue> {
    headerClassName?: string;
    cellClassName?: string;
  }
}

/** Column meta keys supported by {@link DataTable}. */
export type DataTableColumnMeta = Pick<
  import("@tanstack/react-table").ColumnMeta<unknown, unknown>,
  "headerClassName" | "cellClassName"
>;

export interface DataTableProps<TData> {
  /** TanStack column definitions. */
  columns: ColumnDef<TData, unknown>[];
  /** Current page rows. */
  data: TData[];
  /** Optional loading overlay. */
  loading?: boolean;
  /** Shown when `data` is empty and not loading. */
  emptyMessage?: string;
  /** Sticky header inside scroll parent. */
  stickyHeader?: boolean;
  className?: string;
}

/**
 * Render a TanStack Table with shadcn/ui table primitives.
 *
 * @author Xiaoman
 * @created 2026-07-22
 *
 * @param props - columns, data, loading and empty states
 * @returns Table UI
 */
export function DataTable<TData>({
  columns,
  data,
  loading = false,
  emptyMessage,
  stickyHeader = true,
  className,
}: DataTableProps<TData>) {
  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  return (
    <DataTableView
      table={table}
      loading={loading}
      emptyMessage={emptyMessage}
      stickyHeader={stickyHeader}
      className={className}
    />
  );
}

interface DataTableViewProps<TData> {
  table: TanstackTable<TData>;
  loading?: boolean;
  emptyMessage?: string;
  stickyHeader?: boolean;
  className?: string;
}

const stickyHeadClass =
  "sticky top-0 z-10 bg-background shadow-[inset_0_-1px_0_var(--color-border)]";

/**
 * Presentational table view for an existing TanStack table instance.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
export function DataTableView<TData>({
  table,
  loading = false,
  emptyMessage,
  stickyHeader = true,
  className,
}: DataTableViewProps<TData>) {
  const columnCount = table.getAllColumns().length;
  const rows = table.getRowModel().rows;

  return (
    <div
      className={cn(
        "flex min-h-0 flex-1 flex-col overflow-hidden rounded-[var(--radius-md)] border border-border/70",
        className,
      )}
    >
      <div className="min-h-0 flex-1 overflow-auto">
        <Table scrollWrapper={false} className="min-w-[1320px]">
          <TableHeader>
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
                {headerGroup.headers.map((header) => (
                  <TableHead
                    key={header.id}
                    className={cn(
                      stickyHeader ? stickyHeadClass : undefined,
                      header.column.columnDef.meta?.headerClassName,
                    )}
                  >
                    {header.isPlaceholder
                      ? null
                      : flexRender(header.column.columnDef.header, header.getContext())}
                  </TableHead>
                ))}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {rows.length > 0 ? (
              rows.map((row) => (
                <TableRow key={row.id} data-state={row.getIsSelected() ? "selected" : undefined}>
                  {row.getVisibleCells().map((cell) => (
                    <TableCell
                      key={cell.id}
                      className={cell.column.columnDef.meta?.cellClassName}
                    >
                      {flexRender(cell.column.columnDef.cell, cell.getContext())}
                    </TableCell>
                  ))}
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell colSpan={columnCount} className="h-32 text-center text-muted-foreground">
                  {loading ? "…" : emptyMessage}
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  );
}
