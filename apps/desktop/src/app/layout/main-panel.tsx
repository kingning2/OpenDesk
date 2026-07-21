/**
 * 桌面壳主工作区画布。
 *
 * @author coisini
 * @created 2026-07-20
 */

import { cn } from "@desk/ui";
import type { HTMLAttributes, ReactNode } from "react";

/**
 * `MainPanel` 属性。
 *
 * @author coisini
 * @created 2026-07-20
 */
export interface MainPanelProps extends HTMLAttributes<HTMLElement> {
  /** 可选顶部区域。 */
  header?: ReactNode;
  /** 工作区内容。 */
  children?: ReactNode;
}

/**
 * 主面板：shell 底色外框 + workspace 圆角画布。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @param props - 见 {@link MainPanelProps}
 * @returns 主面板节点
 */
export function MainPanel({ header, children, className, ...props }: MainPanelProps) {
  return (
    <main className={cn("flex min-h-0 min-w-0 flex-1 flex-col bg-shell p-2 pl-1.5", className)} {...props}>
      <div className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-[var(--radius-lg)] border border-border bg-workspace shadow-[var(--glass-shadow)]">
        {header}
        <div className="flex min-h-0 flex-1 flex-col overflow-hidden">{children}</div>
      </div>
    </main>
  );
}
