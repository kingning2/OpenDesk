import * as React from "react";

import { cn } from "../../lib/cn";
import { PageContainer } from "./page-container";

export interface PageScaffoldProps extends React.HTMLAttributes<HTMLDivElement> {
  subtitle?: React.ReactNode;
  children?: React.ReactNode;
  containerWidth?: "full" | "lg" | "xl";
  containerPadding?: "none" | "sm" | "md" | "lg";
}

export function PageScaffold({
  subtitle,
  children,
  containerWidth = "lg",
  containerPadding = "md",
  className,
  ...props
}: PageScaffoldProps) {
  return (
    <PageContainer width={containerWidth} padding={containerPadding} className={cn("space-y-4", className)} {...props}>
      {subtitle ? (
        <p className="text-[length:var(--text-sm)] text-muted-foreground">{subtitle}</p>
      ) : null}
      {children}
    </PageContainer>
  );
}
