/**
 * 桌面壳左右分栏布局（侧栏 + 主区）。
 *
 * @author coisini
 * @created 2026-07-20
 */

import { cn } from "@desk/ui";
import type { HTMLAttributes, ReactNode } from "react";

/**
 * `AppLayout` 属性。
 *
 * @author coisini
 * @created 2026-07-20
 */
export interface AppLayoutProps extends HTMLAttributes<HTMLDivElement> {
  /** 左侧侧栏（如 NavRail）。 */
  sidebar?: ReactNode;
  /** 主内容区。 */
  children?: ReactNode;
}

/**
 * 桌面壳主体布局：侧栏与主区水平排列。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @param props - 见 {@link AppLayoutProps}
 * @returns 布局节点
 */
export function AppLayout({ sidebar, children, className, ...props }: AppLayoutProps) {
  return (
    <div className={cn("flex min-h-0 flex-1 flex-col", className)} {...props}>
      <div className="flex min-h-0 flex-1">
        {sidebar}
        {children}
      </div>
    </div>
  );
}
