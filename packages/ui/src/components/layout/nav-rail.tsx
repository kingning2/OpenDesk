import { cva, type VariantProps } from "class-variance-authority";
import * as React from "react";

import { cn } from "../../lib/cn";

export function NavRail({ className, ...props }: React.HTMLAttributes<HTMLElement>) {
  return (
    <aside
      className={cn(
        "flex w-16 shrink-0 flex-col bg-shell py-2",
        className,
      )}
      {...props}
    />
  );
}

export function NavRailNav({ className, ...props }: React.HTMLAttributes<HTMLElement>) {
  return <nav className={cn("flex w-full flex-col gap-0.5 px-1.5", className)} {...props} />;
}

const navRailItemVariants = cva(
  "flex w-full cursor-pointer flex-col items-center gap-1 rounded-[var(--radius-md)] px-1 py-2 text-[10px] leading-none transition-colors",
  {
    variants: {
      active: {
        true: "bg-primary/15 text-primary",
        false: "text-muted-foreground hover:bg-muted/40 hover:text-foreground",
      },
    },
    defaultVariants: {
      active: false,
    },
  },
);

export interface NavRailItemProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof navRailItemVariants> {}

export function NavRailItem({ className, active, ...props }: NavRailItemProps) {
  return <div className={cn(navRailItemVariants({ active }), className)} {...props} />;
}

export { navRailItemVariants };
