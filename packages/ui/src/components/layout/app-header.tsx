import { cva, type VariantProps } from "class-variance-authority";
import * as React from "react";

import { cn } from "../../lib/cn";

const appHeaderVariants = cva("flex shrink-0 items-center border-b border-border", {
  variants: {
    variant: {
      default: "items-start justify-between gap-4 bg-background/80 px-6 py-4 backdrop-blur-[var(--blur-glass)]",
      compact: "h-9 gap-3 border-border bg-transparent px-4 text-[length:var(--text-xs)]",
    },
  },
  defaultVariants: {
    variant: "default",
  },
});

export interface AppHeaderProps
  extends React.HTMLAttributes<HTMLElement>,
    VariantProps<typeof appHeaderVariants> {
  title: string;
  description?: string;
  breadcrumb?: React.ReactNode;
  actions?: React.ReactNode;
}

export function AppHeader({
  title,
  description,
  breadcrumb,
  actions,
  variant,
  className,
  ...props
}: AppHeaderProps) {
  if (variant === "compact") {
    return (
      <header className={cn(appHeaderVariants({ variant }), className)} {...props}>
        <div className="min-w-0 truncate text-muted-foreground">
          {breadcrumb ?? (
            <>
              <span className="text-foreground/80">OpenDesk</span>
              <span className="px-1.5 text-border">/</span>
              <span className="text-foreground">{title}</span>
            </>
          )}
        </div>
        {actions ? <div className="flex shrink-0 items-center gap-2">{actions}</div> : null}
      </header>
    );
  }

  return (
    <header className={cn(appHeaderVariants({ variant }), className)} {...props}>
      <div className="min-w-0 space-y-1">
        {breadcrumb ? (
          <div className="text-[length:var(--text-xs)] text-muted-foreground">{breadcrumb}</div>
        ) : null}
        <h1 className="font-display text-[length:var(--text-xl)] font-semibold leading-tight">{title}</h1>
        {description ? (
          <p className="text-[length:var(--text-sm)] text-muted-foreground">{description}</p>
        ) : null}
      </div>
      {actions ? <div className="flex shrink-0 items-center gap-2">{actions}</div> : null}
    </header>
  );
}

export { appHeaderVariants };

