/**
 * 带无障碍标签的图标按钮。
 *
 * @author coisini
 * @created 2026-07-20
 */

import * as React from "react";

import { cn } from "../lib/cn";

/**
 * `IconButton` 属性。
 *
 * @author coisini
 * @created 2026-07-20
 */
export interface IconButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  /** 无障碍可读名称。 */
  label: string;
}

/**
 * 紧凑图标按钮。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @param props - 见 {@link IconButtonProps}
 * @returns 按钮节点
 */
export function IconButton({ label, className, children, ...props }: IconButtonProps) {
  return (
    <button
      type="button"
      aria-label={label}
      className={cn(
        "inline-flex size-8 cursor-pointer items-center justify-center rounded-[var(--radius-sm)] text-muted-foreground transition-[color,background-color,transform] duration-150 ease-[cubic-bezier(0.23,1,0.32,1)] active:scale-[0.97]",
        "[@media(hover:hover)_and_(pointer:fine)]:hover:bg-muted/60 [@media(hover:hover)_and_(pointer:fine)]:hover:text-foreground",
        className,
      )}
      {...props}
    >
      {children}
    </button>
  );
}
