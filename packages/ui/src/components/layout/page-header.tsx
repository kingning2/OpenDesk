/**
 * 工作区页头 — 标题 / 说明 / 右侧操作区（对齐 ProComponents PageHeader）。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */

import * as React from "react";

import { cn } from "../../lib/cn";

/**
 * 页头属性。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export interface PageHeaderProps extends Omit<React.HTMLAttributes<HTMLDivElement>, "title"> {
  /** 主标题。 */
  title?: React.ReactNode;
  /** 副标题或页面说明。 */
  subtitle?: React.ReactNode;
  /** 右侧操作区（按钮、筛选等）。 */
  extra?: React.ReactNode;
}

/**
 * 工作区页头。
 *
 * @author Xiaoman
 * @created 2026-08-20
 *
 * @param props - 见 {@link PageHeaderProps}
 * @returns 页头节点；无内容时返回 null
 */
export function PageHeader({ title, subtitle, extra, className, ...props }: PageHeaderProps) {
  if (!title && !subtitle && !extra) {
    return null;
  }

  return (
    <div
      className={cn("flex flex-wrap items-start justify-between gap-x-4 gap-y-3", className)}
      {...props}
    >
      <div className="min-w-0 space-y-1">
        {title ? (
          <h1 className="text-[length:var(--text-xl)] font-semibold tracking-tight text-foreground">
            {title}
          </h1>
        ) : null}
        {subtitle ? (
          <p className="text-[length:var(--text-sm)] text-muted-foreground">{subtitle}</p>
        ) : null}
      </div>
      {extra ? (
        <div className="flex shrink-0 flex-wrap items-center justify-end gap-2">{extra}</div>
      ) : null}
    </div>
  );
}
