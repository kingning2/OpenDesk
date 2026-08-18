/**
 * Textarea — 多行文本输入。
 *
 * 与 Input 同风格的受控组件；从原前端多行输入场景抽取。
 */

import * as React from "react";

import { cn } from "../lib/cn";

export type TextareaProps = React.TextareaHTMLAttributes<HTMLTextAreaElement>;

/**
 * 多行文本输入。
 *
 * @author agent
 * @created 2026-08-13
 *
 * @param props - 标准 textarea 属性
 * @returns 文本域节点
 */
export function Textarea({ className, ...props }: TextareaProps) {
  return (
    <textarea
      className={cn(
        "flex min-h-[60px] w-full rounded-[var(--radius-md)] border border-border bg-background px-3 py-2",
        "text-(length:--text-sm) text-foreground outline-none transition-colors",
        "placeholder:text-muted-foreground",
        "focus-visible:ring-2 focus-visible:ring-primary/40",
        "disabled:cursor-not-allowed disabled:opacity-50",
        className,
      )}
      {...props}
    />
  );
}
