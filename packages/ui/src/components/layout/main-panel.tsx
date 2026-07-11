import * as React from "react";

import { cn } from "../../lib/cn";

export interface MainPanelProps extends React.HTMLAttributes<HTMLElement> {
  header?: React.ReactNode;
  children?: React.ReactNode;
}

export function MainPanel({ header, children, className, ...props }: MainPanelProps) {
  return (
    <main className={cn("flex min-h-0 min-w-0 flex-1 flex-col bg-shell p-2 pl-1.5", className)} {...props}>
      <div className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-[var(--radius-lg)] border border-border bg-workspace shadow-[var(--glass-shadow)]">
        {header}
        <div className="flex min-h-0 flex-1 flex-col overflow-hidden">{children}</div>
      </div>
    </main>
  );
}
