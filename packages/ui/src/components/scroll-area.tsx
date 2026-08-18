/**
 * ScrollArea — 自定义滚动条区域（Radix Scroll Area）。
 *
 * 工作区页面与分栏内容应使用本组件，不要写 `overflow-auto`。
 *
 * @author Xiaoman
 * @created 2026-07-20
 */

import * as ScrollAreaPrimitive from "@radix-ui/react-scroll-area";
import * as React from "react";

import { cn } from "../lib/cn";

/**
 * 滚动区域属性。
 *
 * @author Xiaoman
 * @created 2026-08-18
 */
export interface ScrollAreaProps extends React.ComponentProps<typeof ScrollAreaPrimitive.Root> {
  /** 指向真正滚动的 Viewport，用于监听 scroll / scrollTo。 */
  viewportRef?: React.Ref<HTMLDivElement>;
  /** Viewport 额外样式。 */
  viewportClassName?: string;
}

/**
 * 自定义滚动条容器。
 *
 * @author Xiaoman
 * @created 2026-07-20
 *
 * @param props - 见 {@link ScrollAreaProps}
 * @returns 滚动区域节点
 */
export function ScrollArea({
  className,
  children,
  viewportRef,
  viewportClassName,
  ...props
}: ScrollAreaProps) {
  return (
    <ScrollAreaPrimitive.Root className={cn("relative overflow-hidden", className)} {...props}>
      <ScrollAreaPrimitive.Viewport
        ref={viewportRef}
        className={cn("size-full rounded-[inherit]", viewportClassName)}
      >
        {children}
      </ScrollAreaPrimitive.Viewport>
      <ScrollBar />
      <ScrollAreaPrimitive.Corner />
    </ScrollAreaPrimitive.Root>
  );
}

/**
 * 滚动条轨道与滑块。
 *
 * @author Xiaoman
 * @created 2026-07-20
 *
 * @param props - Radix Scrollbar props
 * @returns 滚动条节点
 */
export function ScrollBar({
  className,
  orientation = "vertical",
  ...props
}: React.ComponentProps<typeof ScrollAreaPrimitive.ScrollAreaScrollbar>) {
  return (
    <ScrollAreaPrimitive.ScrollAreaScrollbar
      orientation={orientation}
      className={cn(
        "flex touch-none select-none transition-colors",
        orientation === "vertical" && "h-full w-2.5 border-l border-l-transparent p-px",
        orientation === "horizontal" && "h-2.5 flex-col border-t border-t-transparent p-px",
        className,
      )}
      {...props}
    >
      <ScrollAreaPrimitive.ScrollAreaThumb className="relative flex-1 rounded-full bg-border" />
    </ScrollAreaPrimitive.ScrollAreaScrollbar>
  );
}
