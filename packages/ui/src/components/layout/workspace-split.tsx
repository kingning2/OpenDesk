import * as React from "react";

import { cn } from "../../lib/cn";
import { ScrollArea } from "../scroll-area";

/**
 * 主从分栏工作区容器属性。
 *
 * @author Xiaoman
 * @created 2026-07-21
 */
export interface WorkspaceSplitProps extends React.HTMLAttributes<HTMLDivElement> {
  /** 左侧面板初始宽度（像素）。 */
  defaultStartWidth?: number;
  /** 左侧面板最小宽度（像素）。 */
  minStartWidth?: number;
  /** 左侧面板最大宽度（像素）。 */
  maxStartWidth?: number;
  /** 分栏面板节点。 */
  children?: React.ReactNode;
}

interface WorkspaceSplitContextValue {
  startWidth: number;
}

const WorkspaceSplitContext = React.createContext<WorkspaceSplitContextValue | null>(null);

/**
 * 主从分栏工作区容器。
 *
 * 线框栅格布局：外边框 + 面板间竖线分隔，无嵌套卡片。
 * 配合 `PageScaffold fill containerPadding="none"` 占满工作区。
 *
 * @author Xiaoman
 * @created 2026-07-21
 *
 * @param props - 见 {@link WorkspaceSplitProps}
 * @returns 分栏容器节点
 */
export function WorkspaceSplit({
  className,
  children,
  defaultStartWidth = 300,
  minStartWidth = 240,
  maxStartWidth = 520,
  ...props
}: WorkspaceSplitProps) {
  const [startWidth, setStartWidth] = React.useState(defaultStartWidth);
  const childArray = React.Children.toArray(children);

  const handlePointerDown = React.useCallback(
    (event: React.PointerEvent<HTMLDivElement>) => {
      event.preventDefault();
      const pointerId = event.pointerId;
      const startX = event.clientX;
      const initialWidth = startWidth;
      const target = event.currentTarget;

      target.setPointerCapture(pointerId);
      document.body.style.cursor = "col-resize";
      document.body.style.userSelect = "none";

      const handlePointerMove = (moveEvent: PointerEvent) => {
        const delta = moveEvent.clientX - startX;
        const nextWidth = Math.min(maxStartWidth, Math.max(minStartWidth, initialWidth + delta));
        setStartWidth(nextWidth);
      };

      const cleanup = () => {
        document.body.style.cursor = "";
        document.body.style.userSelect = "";
        target.removeEventListener("pointermove", handlePointerMove);
        target.removeEventListener("pointerup", cleanup);
        target.removeEventListener("pointercancel", cleanup);
        if (target.hasPointerCapture(pointerId)) {
          target.releasePointerCapture(pointerId);
        }
      };

      target.addEventListener("pointermove", handlePointerMove);
      target.addEventListener("pointerup", cleanup);
      target.addEventListener("pointercancel", cleanup);
    },
    [maxStartWidth, minStartWidth, startWidth],
  );

  return (
    <WorkspaceSplitContext.Provider value={{ startWidth }}>
      <div
        className={cn(
          "flex min-h-0 flex-1 overflow-hidden border border-border/70 bg-background/30",
          className,
        )}
        {...props}
      >
        {childArray.map((child, index) => (
          <React.Fragment key={index}>
            {child}
            {index === 0 && childArray.length > 1 ? (
              <div
                role="separator"
                aria-orientation="vertical"
                aria-label="Resize panels"
                onPointerDown={handlePointerDown}
                className={cn(
                  "relative w-2 shrink-0 cursor-col-resize bg-transparent",
                  "before:absolute before:bottom-0 before:left-1/2 before:top-0 before:w-px before:-translate-x-1/2 before:bg-border/70",
                  "after:absolute after:bottom-0 after:left-1/2 after:top-0 after:w-[3px] after:-translate-x-1/2 after:bg-transparent",
                  "hover:after:bg-primary/20",
                )}
              />
            ) : null}
          </React.Fragment>
        ))}
      </div>
    </WorkspaceSplitContext.Provider>
  );
}

const paneWidthClass = {
  sm: "w-[300px]",
  md: "w-[360px]",
} as const;

/**
 * 主从分栏单面板属性。
 *
 * @author Xiaoman
 * @created 2026-07-21
 */
export interface WorkspaceSplitPaneProps extends React.HTMLAttributes<HTMLDivElement> {
  /** 固定头部区域（标题、操作、筛选等）。 */
  header?: React.ReactNode;
  /** `start` 为固定侧栏，`main` 为自适应主面板。 */
  side?: "start" | "main";
  /** 侧栏宽度，仅在 `side="start"` 时生效。 */
  width?: keyof typeof paneWidthClass;
  /** 内容区是否可滚动，默认 `true`。 */
  scroll?: boolean;
  /** 面板主体内容。 */
  children?: React.ReactNode;
}

/**
 * 主从分栏单面板。
 *
 * 竖线/横线分隔的扁平面板，头部固定、内容区可滚动。
 *
 * @author Xiaoman
 * @created 2026-07-21
 *
 * @param props - 见 {@link WorkspaceSplitPaneProps}
 * @returns 分栏面板节点
 */
export function WorkspaceSplitPane({
  header,
  side = "main",
  width = "sm",
  scroll = true,
  className,
  children,
  ...props
}: WorkspaceSplitPaneProps) {
  const splitContext = React.useContext(WorkspaceSplitContext);
  const body = scroll ? (
    <ScrollArea className="min-h-0 flex-1">{children}</ScrollArea>
  ) : (
    <div className="min-h-0 flex-1 overflow-hidden">{children}</div>
  );

  return (
    <div
      className={cn(
        "flex min-h-0 flex-col overflow-hidden",
        side === "start" && cn("shrink-0", !splitContext && paneWidthClass[width]),
        side === "main" && "min-w-0 flex-1",
        className,
      )}
      style={
        side === "start" && splitContext
          ? { width: `${splitContext.startWidth}px` }
          : undefined
      }
      {...props}
    >
      {header ? <div className="shrink-0 border-b border-border/70">{header}</div> : null}
      {body}
    </div>
  );
}

/**
 * 分栏面板工具栏属性。
 *
 * @author Xiaoman
 * @created 2026-07-21
 */
export interface WorkspaceSplitToolbarProps extends React.HTMLAttributes<HTMLDivElement> {
  children?: React.ReactNode;
}

/**
 * 分栏面板工具栏行。
 *
 * @author Xiaoman
 * @created 2026-07-21
 *
 * @param props - 见 {@link WorkspaceSplitToolbarProps}
 * @returns 工具栏节点
 */
export function WorkspaceSplitToolbar({ className, children, ...props }: WorkspaceSplitToolbarProps) {
  return (
    <div className={cn("flex items-center gap-3 px-4 py-3", className)} {...props}>
      {children}
    </div>
  );
}

/**
 * 分栏面板标题属性。
 *
 * @author Xiaoman
 * @created 2026-07-21
 */
export interface WorkspaceSplitTitleProps extends React.HTMLAttributes<HTMLHeadingElement> {
  children?: React.ReactNode;
}

/**
 * 分栏面板标题。
 *
 * @author Xiaoman
 * @created 2026-07-21
 *
 * @param props - 见 {@link WorkspaceSplitTitleProps}
 * @returns 标题节点
 */
export function WorkspaceSplitTitle({ className, children, ...props }: WorkspaceSplitTitleProps) {
  return (
    <h2
      className={cn(
        "font-display text-[length:var(--text-sm)] font-semibold tracking-tight text-foreground",
        className,
      )}
      {...props}
    >
      {children}
    </h2>
  );
}

/**
 * 分栏列表面板行属性。
 *
 * @author Xiaoman
 * @created 2026-07-21
 */
export interface WorkspaceSplitRowProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  /** 是否为当前选中行。 */
  active?: boolean;
}

/**
 * 分栏列表面板行。
 *
 * 横线分隔的扁平列表项，用于主从布局左侧列表。
 *
 * @author Xiaoman
 * @created 2026-07-21
 *
 * @param props - 见 {@link WorkspaceSplitRowProps}
 * @returns 列表行节点
 */
export function WorkspaceSplitRow({
  active = false,
  className,
  children,
  type = "button",
  ...props
}: WorkspaceSplitRowProps) {
  return (
    <button
      type={type}
      className={cn(
        "w-full border-b border-border/50 px-4 py-2.5 text-left transition-colors",
        "duration-150 ease-[cubic-bezier(0.23,1,0.32,1)]",
        active
          ? "bg-primary/10 text-foreground"
          : "text-foreground [@media(hover:hover)_and_(pointer:fine)]:hover:bg-muted/40",
        className,
      )}
      {...props}
    >
      {children}
    </button>
  );
}

/**
 * 线框表单栅格属性。
 *
 * @author Xiaoman
 * @created 2026-07-21
 */
export interface WorkspaceSplitGridProps extends React.HTMLAttributes<HTMLDivElement> {
  /** 栅格列数，默认 `2`。 */
  columns?: 1 | 2;
  children?: React.ReactNode;
}

/**
 * 线框表单栅格容器。
 *
 * 用横竖线分隔字段单元格，替代嵌套卡片式表单布局。
 *
 * @author Xiaoman
 * @created 2026-07-21
 *
 * @param props - 见 {@link WorkspaceSplitGridProps}
 * @returns 栅格容器节点
 */
export function WorkspaceSplitGrid({
  columns = 2,
  className,
  children,
  ...props
}: WorkspaceSplitGridProps) {
  return (
    <div
      className={cn(
        "grid border-t border-border/50",
        columns === 2 && "md:grid-cols-2",
        className,
      )}
      {...props}
    >
      {children}
    </div>
  );
}

/**
 * 线框表单栅格单元属性。
 *
 * @author Xiaoman
 * @created 2026-07-21
 */
export interface WorkspaceSplitCellProps extends React.HTMLAttributes<HTMLDivElement> {
  /** 是否占满整行（跨列）。 */
  span?: "full";
  children?: React.ReactNode;
}

/**
 * 线框表单栅格单元。
 *
 * @author Xiaoman
 * @created 2026-07-21
 *
 * @param props - 见 {@link WorkspaceSplitCellProps}
 * @returns 栅格单元节点
 */
export function WorkspaceSplitCell({ span, className, children, ...props }: WorkspaceSplitCellProps) {
  return (
    <div
      className={cn(
        "border-b border-border/50 p-3",
        span !== "full" && "md:border-r md:[&:nth-child(2n)]:border-r-0",
        span === "full" && "md:col-span-2",
        className,
      )}
      {...props}
    >
      {children}
    </div>
  );
}
