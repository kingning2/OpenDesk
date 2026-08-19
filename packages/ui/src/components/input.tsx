/**
 * Input — shadcn 风格单行输入。
 *
 * FormInput 的视觉叶子；也可在 Feature 里单独受控使用。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */

import * as React from "react";

import { cn } from "../lib/cn";

/**
 * 单行输入属性（标准 `input`，含 React 19 `ref`）。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */
export type InputProps = React.ComponentProps<"input">;

/**
 * 单行输入。
 *
 * @author Xiaoman
 * @created 2026-08-19
 *
 * @param props - 见 {@link InputProps}
 * @returns 输入框节点
 */
export function Input({ className, type = "text", ...props }: InputProps) {
  return (
    <input
      type={type}
      className={cn(
        "flex h-9 w-full rounded-md border border-border bg-background px-3",
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
