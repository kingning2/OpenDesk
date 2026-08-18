/**
 * 工作区页面基础容器 — 占满右侧 MainPanel，内容超出时用 ScrollArea 滚动。
 *
 * Feature 页应包在本组件内，不要在页面里写 `overflow-auto`。
 * 分栏页若自行管理内部滚动，传 `scroll={false}`。
 *
 * @author Xiaoman
 * @created 2026-07-20
 */

import * as React from "react";

import { cn } from "../../lib/cn";
import { ScrollArea } from "../scroll-area";
import { PageContainer } from "./page-container";

/**
 * 工作区页面容器属性。
 *
 * @author Xiaoman
 * @created 2026-07-20
 */
export interface PageScaffoldProps extends React.HTMLAttributes<HTMLDivElement> {
  /** 页面说明，渲染在内容顶部。 */
  subtitle?: React.ReactNode;
  /** 固定在滚动区外的页头（工具栏等）。 */
  header?: React.ReactNode;
  /** 页面主体。 */
  children?: React.ReactNode;
  /** 内容区最大宽度。 */
  containerWidth?: "full" | "lg" | "xl";
  /** 内容区内边距。 */
  containerPadding?: "none" | "sm" | "md" | "lg";
  /**
   * 占满主面板（flex 列）。
   *
   * @default true
   */
  fill?: boolean;
  /**
   * 内容超出时用 ScrollArea 滚动。
   * 分栏自管滚动时设为 `false`。
   *
   * @default true
   */
  scroll?: boolean;
}

/**
 * 工作区页面基础容器。
 *
 * 负责：
 * - 占满右侧工作区画布
 * - 用 ScrollArea 展示纵向滚动条（可关闭）
 * - 统一内边距与内容宽度
 *
 * @author Xiaoman
 * @created 2026-07-20
 *
 * @param props - 见 {@link PageScaffoldProps}
 * @returns 页面容器节点
 */
export function PageScaffold({
  subtitle,
  header,
  children,
  containerWidth = "full",
  containerPadding = "md",
  fill = true,
  scroll = true,
  className,
  ...props
}: PageScaffoldProps) {
  const body = (
    <PageContainer
      width={containerWidth}
      padding={containerPadding}
      className={cn("space-y-4", !scroll && "flex h-full min-h-0 flex-1 flex-col", className)}
      {...props}
    >
      {subtitle ? (
        <p className="text-(length:--text-sm) text-muted-foreground">{subtitle}</p>
      ) : null}
      {children}
    </PageContainer>
  );

  return (
    <div className={cn("flex min-h-0 flex-col overflow-hidden", fill && "flex-1")}>
      {header ? <div className="shrink-0">{header}</div> : null}
      {scroll ? (
        <ScrollArea className="min-h-0 flex-1">{body}</ScrollArea>
      ) : (
        <div className="flex min-h-0 flex-1 flex-col overflow-hidden">{body}</div>
      )}
    </div>
  );
}
