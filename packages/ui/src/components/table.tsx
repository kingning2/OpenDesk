/**
 * Table — shadcn 表格原语。
 *
 * 只负责语义结构与视觉；列定义 / 排序 / 分页交给 DataTable。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */

import * as React from "react";

import { cn } from "../lib/cn";

/**
 * 表格根（外层滚动容器 + `table`）。
 *
 * @author Xiaoman
 * @created 2026-08-19
 *
 * @param props - 标准 `table` 属性
 * @returns 表格节点
 */
export function Table({ className, ...props }: React.ComponentProps<"table">) {
  return (
    <div className="relative w-full overflow-x-auto rounded-xl border border-border">
      <table
        className={cn("w-full caption-bottom text-(length:--text-sm)", className)}
        {...props}
      />
    </div>
  );
}

/**
 * 表头。
 *
 * @author Xiaoman
 * @created 2026-08-19
 *
 * @param props - 标准 `thead` 属性
 * @returns 表头节点
 */
export function TableHeader({ className, ...props }: React.ComponentProps<"thead">) {
  return <thead className={cn("bg-muted/50 text-muted-foreground", className)} {...props} />;
}

/**
 * 表体。
 *
 * @author Xiaoman
 * @created 2026-08-19
 *
 * @param props - 标准 `tbody` 属性
 * @returns 表体节点
 */
export function TableBody({ className, ...props }: React.ComponentProps<"tbody">) {
  return <tbody className={cn("divide-y divide-border", className)} {...props} />;
}

/**
 * 表尾行。
 *
 * @author Xiaoman
 * @created 2026-08-19
 *
 * @param props - 标准 `tfoot` 属性
 * @returns 表脚节点
 */
export function TableFooter({ className, ...props }: React.ComponentProps<"tfoot">) {
  return (
    <tfoot
      className={cn("border-t border-border bg-muted/50 font-medium", className)}
      {...props}
    />
  );
}

/**
 * 表格行。
 *
 * @author Xiaoman
 * @created 2026-08-19
 *
 * @param props - 标准 `tr` 属性
 * @returns 行节点
 */
export function TableRow({ className, ...props }: React.ComponentProps<"tr">) {
  return (
    <tr
      className={cn(
        "transition-colors duration-150 ease-[cubic-bezier(0.23,1,0.32,1)]",
        "[@media(hover:hover)_and_(pointer:fine)]:hover:bg-muted/30",
        "data-[state=selected]:bg-muted/40",
        className,
      )}
      {...props}
    />
  );
}

/**
 * 列表头单元格。
 *
 * @author Xiaoman
 * @created 2026-08-19
 *
 * @param props - 标准 `th` 属性
 * @returns 表头单元格
 */
export function TableHead({ className, ...props }: React.ComponentProps<"th">) {
  return (
    <th
      className={cn(
        "px-4 py-2.5 text-left align-middle font-medium whitespace-nowrap",
        "[&:has([role=checkbox])]:pr-0",
        className,
      )}
      {...props}
    />
  );
}

/**
 * 数据单元格。
 *
 * @author Xiaoman
 * @created 2026-08-19
 *
 * @param props - 标准 `td` 属性
 * @returns 单元格节点
 */
export function TableCell({ className, ...props }: React.ComponentProps<"td">) {
  return (
    <td
      className={cn("px-4 py-2.5 align-middle [&:has([role=checkbox])]:pr-0", className)}
      {...props}
    />
  );
}

/**
 * 表格标题（可选）。
 *
 * @author Xiaoman
 * @created 2026-08-19
 *
 * @param props - 标准 `caption` 属性
 * @returns 标题节点
 */
export function TableCaption({ className, ...props }: React.ComponentProps<"caption">) {
  return (
    <caption
      className={cn("mt-3 text-(length:--text-sm) text-muted-foreground", className)}
      {...props}
    />
  );
}
