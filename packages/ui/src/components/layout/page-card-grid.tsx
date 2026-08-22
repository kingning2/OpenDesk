/**
 * 工作区卡片列表栅格 — 按剩余宽度决定列数（最多 3 列），同行卡片等高。
 *
 * 使用 `auto-fill`（非 `auto-fit`）：条目少时不把两张卡拉成半宽，宽屏可稳定到三列。
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
        // 列宽 ≥ max(17.5rem, 约 1/3 容器)，故最多 3 列；更窄自动 2/1 列。
        // auto-fill 保留空轨，避免仅 2 张卡时被拉成各占一半。
        "grid grid-cols-[repeat(auto-fill,minmax(min(100%,max(17.5rem,calc((100%-2rem)/3))),1fr))] items-stretch gap-4 [&>*]:h-full [&>*]:min-h-0",
        className,
      )}
      {...props}
    />
  );
}
