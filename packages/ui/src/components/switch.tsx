/**
 * Switch — 开关。
 *
 * 用 button[role=switch] 封装，避免 Feature 手写原生 checkbox + peer 样式。
 *
 * @author Xiaoman
 * @created 2026-08-18
 */

import * as React from "react";

import { cn } from "../lib/cn";

/**
 * 开关属性。
 *
 * @author Xiaoman
 * @created 2026-08-18
 */
export interface SwitchProps
  extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "onChange"> {
  /** 是否开启。 */
  checked?: boolean;
  /** 开关变化回调。 */
  onCheckedChange?: (checked: boolean) => void;
}

/**
 * 开关。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @param props - 见 {@link SwitchProps}
 * @returns 开关节点
 */
export function Switch({
  checked = false,
  onCheckedChange,
  className,
  disabled,
  ...props
}: SwitchProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      disabled={disabled}
      onClick={() => onCheckedChange?.(!checked)}
      className={cn(
        "relative inline-flex h-5 w-9 shrink-0 items-center rounded-full",
        "transition-[background-color,transform] duration-150 ease-[cubic-bezier(0.23,1,0.32,1)]",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/40",
        "disabled:pointer-events-none disabled:opacity-50",
        checked ? "bg-primary" : "bg-border",
        className,
      )}
      {...props}
    >
      <span
        className={cn(
          "pointer-events-none block size-4 rounded-full bg-background shadow-sm",
          "transition-transform duration-150 ease-[cubic-bezier(0.23,1,0.32,1)]",
          checked ? "translate-x-4" : "translate-x-0.5",
        )}
      />
    </button>
  );
}
