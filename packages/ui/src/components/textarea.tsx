/**
 * Textarea — 多行文本输入。
 *
 * 与 Input 同风格；FormTextarea 的视觉叶子。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */

import * as React from "react";

import { cn } from "../lib/cn";

/**
 * 多行输入属性（标准 `textarea`，含 React 19 `ref`）。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */
export type TextareaProps = React.ComponentProps<"textarea">;

/**
 * 多行文本输入。
 *
 * @author Xiaoman
 * @created 2026-08-19
 *
 * @param props - 见 {@link TextareaProps}
 * @returns 文本域节点
 */
export function Textarea({ className, ...props }: TextareaProps) {
  return (
    <textarea
      className={cn(
        "flex min-h-[60px] w-full rounded-[var(--radius-md)] border border-border bg-background px-3 py-2",
        "text-(length:--text-sm) text-foreground outline-none",
        "transition-[border-color,box-shadow] duration-150 ease-[cubic-bezier(0.23,1,0.32,1)]",
        "placeholder:text-muted-foreground",
        "focus-visible:ring-2 focus-visible:ring-primary/40",
        "disabled:cursor-not-allowed disabled:opacity-50",
        className,
      )}
      {...props}
    />
  );
}
