import { cva, type VariantProps } from "class-variance-authority";
import * as React from "react";

import { cn } from "../../lib/cn";

const sidebarVariants = cva("flex shrink-0 flex-col border-r border-border", {
  variants: {
    variant: {
      default: "bg-muted/30",
      glass: [
        "border-[color:var(--glass-border)]",
        "bg-[color:var(--glass-bg)]",
        "backdrop-blur-[var(--blur-glass)]",
      ].join(" "),
    },
    width: {
      sm: "w-44",
      md: "w-52",
      lg: "w-60",
    },
  },
  defaultVariants: {
    variant: "glass",
    width: "md",
  },
});

export interface SidebarProps
  extends React.HTMLAttributes<HTMLElement>,
    VariantProps<typeof sidebarVariants> {}

export function Sidebar({ className, variant, width, ...props }: SidebarProps) {
  return <aside className={cn(sidebarVariants({ variant, width }), className)} {...props} />;
}

export function SidebarHeader({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn(
        "border-b border-border px-4 py-3 font-display text-[length:var(--text-sm)] font-semibold",
        className,
      )}
      {...props}
    />
  );
}

export function SidebarContent({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn("flex flex-1 flex-col p-3", className)} {...props} />;
}

export { sidebarVariants };
