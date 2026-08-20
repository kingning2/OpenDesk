/**
 * 带无障碍标签的图标按钮（基于 {@link Button}）。
 *
 * @author coisini
 * @created 2026-07-20
 */

import * as React from "react";

import { cn } from "../lib/cn";
import { Button } from "./button";

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
    <Button
      type="button"
      variant="ghost"
      size="icon"
      aria-label={label}
      className={cn("size-8 text-muted-foreground", className)}
      {...props}
    >
      {children}
    </Button>
  );
}
