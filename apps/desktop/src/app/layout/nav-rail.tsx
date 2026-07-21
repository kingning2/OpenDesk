/**
 * 桌面壳窄轨导航（NavRail）。
 *
 * @author coisini
 * @created 2026-07-20
 */

import { cn } from "@desk/ui";
import { cva, type VariantProps } from "class-variance-authority";
import type { HTMLAttributes } from "react";

/**
 * 窄轨侧栏容器。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @param props - 标准 aside 属性
 * @returns 侧栏节点
 */
export function NavRail({ className, ...props }: HTMLAttributes<HTMLElement>) {
  return (
    <aside
      className={cn("flex w-16 shrink-0 flex-col bg-shell py-2", className)}
      {...props}
    />
  );
}

/**
 * 窄轨内导航列表容器。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @param props - 标准 nav 属性
 * @returns 导航节点
 */
export function NavRailNav({ className, ...props }: HTMLAttributes<HTMLElement>) {
  return <nav className={cn("flex w-full flex-col gap-0.5 px-1.5", className)} {...props} />;
}

/** 窄轨导航项样式变体。 */
const navRailItemVariants = cva(
  "flex w-full cursor-pointer flex-col items-center gap-1 rounded-[var(--radius-md)] px-1 py-2 text-[10px] leading-none transition-[color,background-color,transform] duration-150 ease-[cubic-bezier(0.23,1,0.32,1)] active:scale-[0.97]",
  {
    variants: {
      active: {
        true: "bg-primary/15 text-primary",
        false: "text-muted-foreground hover:bg-muted/40 hover:text-foreground",
      },
    },
    defaultVariants: {
      active: false,
    },
  },
);

/**
 * `NavRailItem` 属性。
 *
 * @author coisini
 * @created 2026-07-20
 */
export interface NavRailItemProps
  extends HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof navRailItemVariants> {}

/**
 * 窄轨导航项（非路由场景可用；路由场景可只用 `navRailItemVariants`）。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @param props - 见 {@link NavRailItemProps}
 * @returns 导航项节点
 */
export function NavRailItem({ className, active, ...props }: NavRailItemProps) {
  return <div className={cn(navRailItemVariants({ active }), className)} {...props} />;
}

export { navRailItemVariants };
