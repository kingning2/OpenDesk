import { cva, type VariantProps } from "class-variance-authority";
import * as React from "react";

import { cn } from "../../lib/cn";

export function SidebarNav({ className, ...props }: React.HTMLAttributes<HTMLElement>) {
  return <nav className={cn("flex flex-col gap-1", className)} {...props} />;
}

const sidebarNavItemVariants = cva(
  "rounded-[var(--radius-md)] px-3 py-2 text-[length:var(--text-sm)] transition-colors cursor-pointer",
  {
    variants: {
      active: {
        true: "bg-primary text-primary-foreground",
        false: "text-muted-foreground hover:bg-muted/60 hover:text-foreground",
      },
    },
    defaultVariants: {
      active: false,
    },
  },
);

export interface SidebarNavItemProps
  extends React.AnchorHTMLAttributes<HTMLAnchorElement>,
    VariantProps<typeof sidebarNavItemVariants> {}

export function SidebarNavItem({ className, active, ...props }: SidebarNavItemProps) {
  return <a className={cn(sidebarNavItemVariants({ active }), className)} {...props} />;
}

export { sidebarNavItemVariants };
