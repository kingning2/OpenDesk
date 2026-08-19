/**
 * Label — 表单字段标签。
 *
 * 绑定 `htmlFor` 指向控件 id；FormInput 内部使用，也可单独组合。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */

import * as React from "react";

import { cn } from "../lib/cn";

/**
 * 标签属性。
 *
 * @author Xiaoman
 * @created 2026-08-19
 */
export type LabelProps = React.ComponentProps<"label">;

/**
 * 表单字段标签。
 *
 * @author Xiaoman
 * @created 2026-08-19
 *
 * @param props - 见 {@link LabelProps}
 * @returns 标签节点
 */
export function Label({ className, ...props }: LabelProps) {
  return (
    <label
      className={cn(
        "text-(length:--text-sm) leading-none text-muted-foreground",
        "peer-disabled:cursor-not-allowed peer-disabled:opacity-50",
        className,
      )}
      {...props}
    />
  );
}
