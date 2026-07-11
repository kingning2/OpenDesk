import * as React from "react";

import { cn } from "../../lib/cn";

export interface AppLayoutProps extends React.HTMLAttributes<HTMLDivElement> {
  sidebar?: React.ReactNode;
  children?: React.ReactNode;
}

export function AppLayout({ sidebar, children, className, ...props }: AppLayoutProps) {
  return (
    <div className={cn("flex min-h-0 flex-1 flex-col", className)} {...props}>
      <div className="flex min-h-0 flex-1">
        {sidebar}
        {children}
      </div>
    </div>
  );
}
