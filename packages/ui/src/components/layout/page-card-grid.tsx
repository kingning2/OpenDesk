/**
 * 工作区卡片列表栅格 — `auto-fit` 自适应列宽，条目少时卡片仍铺满可用宽度。
 *
 * 列表页（账号、商品等）统一使用本组件，勿在各 Feature 重复写 grid 类名。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */

import * as React from "react";

import { cn } from "../../lib/cn";

/**
 * 卡片列表栅格属性。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export type PageCardGridProps = React.HTMLAttributes<HTMLDivElement>;

/**
 * 工作区卡片列表栅格。
 *
 * @author Xiaoman
 * @created 2026-08-20
 *
 * @param props - 标准 div 属性
 * @returns 栅格容器节点
 */
export function PageCardGrid({ className, ...props }: PageCardGridProps) {
  return (
    <div
      className={cn(
        "grid grid-cols-[repeat(auto-fit,minmax(min(100%,17.5rem),1fr))] gap-4",
        className,
      )}
      {...props}
    />
  );
}
