/**
 * shadcn/ui Table primitives — semantic table layout for TanStack Table.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */

import * as React from "react";

import { cn } from "../lib/cn";

export interface TableProps extends React.ComponentProps<"table"> {
  /** Wrap table in a horizontal scroll container (disable when parent handles scroll). */
  scrollWrapper?: boolean;
}

/**
 * Scrollable table container.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
export function Table({ className, scrollWrapper = true, ...props }: TableProps) {
  const table = (
    <table
      data-slot="table"
      className={cn("min-w-full caption-bottom text-[length:var(--text-sm)] w-full", className)}
      {...props}
    />
  );

  if (!scrollWrapper) {
    return table;
  }

  return (
    <div data-slot="table-container" className="relative w-full overflow-x-auto">
      {table}
    </div>
  );
}

/**
 * Table header section.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
export function TableHeader({ className, ...props }: React.ComponentProps<"thead">) {
  return (
    <thead
      data-slot="table-header"
      className={cn("[&_tr]:border-b [&_tr]:border-border/70", className)}
      {...props}
    />
  );
}

/**
 * Table body section.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
export function TableBody({ className, ...props }: React.ComponentProps<"tbody">) {
  return (
    <tbody
      data-slot="table-body"
      className={cn("[&_tr:last-child]:border-0", className)}
      {...props}
    />
  );
}

/**
 * Table footer section.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
export function TableFooter({ className, ...props }: React.ComponentProps<"tfoot">) {
  return (
    <tfoot
      data-slot="table-footer"
      className={cn(
        "border-t border-border/70 bg-muted/40 font-medium [&>tr]:last:border-b-0",
        className,
      )}
      {...props}
    />
  );
}

/**
 * Table row.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
export function TableRow({ className, ...props }: React.ComponentProps<"tr">) {
  return (
    <tr
      data-slot="table-row"
      className={cn(
        "border-b border-border/40 transition-colors [@media(hover:hover)_and_(pointer:fine)]:hover:bg-muted/30 data-[state=selected]:bg-muted",
        className,
      )}
      {...props}
    />
  );
}

/**
 * Table head cell.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
export function TableHead({ className, ...props }: React.ComponentProps<"th">) {
  return (
    <th
      data-slot="table-head"
      className={cn(
        "h-9 whitespace-nowrap px-4 text-left align-middle text-[length:var(--text-xs)] font-medium text-muted-foreground",
        "[&:has([role=checkbox])]:pr-0",
        className,
      )}
      {...props}
    />
  );
}

/**
 * Table body cell.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
export function TableCell({ className, ...props }: React.ComponentProps<"td">) {
  return (
    <td
      data-slot="table-cell"
      className={cn("whitespace-nowrap px-4 py-2 align-middle [&:has([role=checkbox])]:pr-0", className)}
      {...props}
    />
  );
}

/**
 * Table caption.
 *
 * @author Xiaoman
 * @created 2026-07-22
 */
export function TableCaption({ className, ...props }: React.ComponentProps<"caption">) {
  return (
    <caption
      data-slot="table-caption"
      className={cn("mt-4 text-[length:var(--text-sm)] text-muted-foreground", className)}
      {...props}
    />
  );
}
