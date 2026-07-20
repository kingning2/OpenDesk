import * as React from "react";

import { cn } from "../../lib/cn";
import { PageContainer } from "./page-container";

export interface PageScaffoldProps extends React.HTMLAttributes<HTMLDivElement> {
  subtitle?: React.ReactNode;
  children?: React.ReactNode;
  containerWidth?: "full" | "lg" | "xl";
  containerPadding?: "none" | "sm" | "md" | "lg";
  /** Stretch to fill the main panel (flex column). */
  fill?: boolean;
}

export function PageScaffold({
  subtitle,
  children,
  containerWidth = "lg",
  containerPadding = "md",
  fill = false,
  className,
  ...props
}: PageScaffoldProps) {
  return (
    <PageContainer
      width={containerWidth}
      padding={containerPadding}
      className={cn(
        "space-y-4",
        fill && "flex min-h-0 flex-1 flex-col",
        className,
      )}
      {...props}
    >
      {subtitle ? (
        <p className="text-(length:--text-sm) text-muted-foreground">{subtitle}</p>
      ) : null}
      {children}
    </PageContainer>
  );
}
