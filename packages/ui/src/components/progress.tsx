/**
 * Progress — 进度条（Radix Progress 二次封装）。
 *
 * 业务 UI 需要进度展示时优先使用本组件，不要手写 div 进度条。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */

import * as ProgressPrimitive from "@radix-ui/react-progress";
import * as React from "react";

import { cn } from "../lib/cn";

/**
 * 进度条属性。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */
export interface ProgressProps extends React.ComponentPropsWithoutRef<typeof ProgressPrimitive.Root> {
  /** 0–100；不传时为 indeterminate（无确定进度）。 */
  value?: number;
}

/**
 * 水平进度条。
 *
 * @author Xiaoman
 * @created 2026-08-19
 *
 * @param props - 见 {@link ProgressProps}
 * @returns 进度条节点
 */
export const Progress = React.forwardRef<
  React.ElementRef<typeof ProgressPrimitive.Root>,
  ProgressProps
>(({ className, value, ...props }, ref) => (
  <ProgressPrimitive.Root
    ref={ref}
    className={cn("relative h-1.5 w-full overflow-hidden rounded-full bg-muted", className)}
    {...props}
    value={value}
  >
    <ProgressPrimitive.Indicator
      className="size-full flex-1 bg-primary transition-transform duration-200 ease-out"
      style={{ transform: `translateX(-${100 - (value ?? 0)}%)` }}
    />
  </ProgressPrimitive.Root>
));
Progress.displayName = ProgressPrimitive.Root.displayName;
