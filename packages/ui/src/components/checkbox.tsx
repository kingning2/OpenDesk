/**
 * Checkbox — 勾选框。
 *
 * 用 button + lucide Check 封装，避免 Feature 手写原生 `input[type=checkbox]`。
 *
 * @author Xiaoman
 * @created 2026-08-18
 */

import * as React from "react";

import { Check } from "../icons";
import { cn } from "../lib/cn";

/**
 * 勾选框属性。
 *
 * @author Xiaoman
 * @created 2026-08-18
 */
export interface CheckboxProps
  extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "onChange"> {
  /** 是否勾选。 */
  checked?: boolean;
  /** 勾选变化回调。 */
  onCheckedChange?: (checked: boolean) => void;
}

/**
 * 勾选框。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @param props - 见 {@link CheckboxProps}
 * @returns 勾选框节点
 */
export function Checkbox({
  checked = false,
  onCheckedChange,
  className,
  disabled,
  ...props
}: CheckboxProps) {
  return (
    <button
      type="button"
      role="checkbox"
      aria-checked={checked}
      disabled={disabled}
      onClick={() => onCheckedChange?.(!checked)}
      className={cn(
        "inline-flex size-4 shrink-0 items-center justify-center rounded-[var(--radius-sm)] border border-border",
        "transition-[background-color,border-color,transform] duration-150 ease-[cubic-bezier(0.23,1,0.32,1)]",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/40",
        "disabled:pointer-events-none disabled:opacity-50 active:scale-[0.97]",
        checked ? "border-primary bg-primary text-primary-foreground" : "bg-background",
        className,
      )}
      {...props}
    >
      {checked ? <Check className="size-3" aria-hidden /> : null}
    </button>
  );
}
