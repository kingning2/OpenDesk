/**
 * OpenDesk 桌面壳窗口标题栏（无边框窗口拖拽 + 系统窗口控制）。
 *
 * 仅供 `apps/desktop` 壳层使用，不放入 `@desk/ui`。
 *
 * @author Xiaoman
 * @created 2026-07-20
 */

import { cn } from "@desk/ui";
import { Minus, Square, X } from "@desk/ui/icons";
import type { HTMLAttributes, ReactNode, MouseEvent } from "react";

/** 标题栏适配的桌面平台。 */
export type TitleBarPlatform = "macos" | "windows" | "linux";

/**
 * 窗口标题栏属性。
 *
 * @author Xiaoman
 * @created 2026-07-20
 */
export interface TitleBarProps extends HTMLAttributes<HTMLElement> {
  /** 桌面平台；决定流量灯占位或 Windows 窗口按钮。 */
  platform?: TitleBarPlatform;
  /** 中间标签区（如 TabBar）。 */
  tabs?: ReactNode;
  /** 右侧操作区（设置、主题等）。 */
  actions?: ReactNode;
  /** 开始拖拽窗口。 */
  onStartDrag?: () => void;
  /** 最小化。 */
  onMinimize?: () => void;
  /** 切换最大化。 */
  onToggleMaximize?: () => void;
  /** 关闭窗口。 */
  onClose?: () => void;
}

/**
 * 窗口控制按钮。
 *
 * @author Xiaoman
 * @created 2026-07-20
 *
 * @param props.label - 无障碍标签
 * @param props.onClick - 点击回调
 * @param props.children - 图标
 * @param props.className - 额外样式
 * @returns 按钮节点
 */
function WindowControlButton({
  label,
  onClick,
  children,
  className,
}: {
  label: string;
  onClick?: () => void;
  children: ReactNode;
  className?: string;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      onClick={onClick}
      className={cn(
        "inline-flex h-8 w-9 cursor-pointer items-center justify-center text-muted-foreground transition-[color,background-color,transform] duration-150 ease-[cubic-bezier(0.23,1,0.32,1)] hover:bg-muted/60 hover:text-foreground active:scale-[0.97]",
        className,
      )}
    >
      {children}
    </button>
  );
}

/**
 * Windows / Linux 窗口控制组。
 *
 * @author Xiaoman
 * @created 2026-07-20
 *
 * @param props - 最小化 / 最大化 / 关闭回调
 * @returns 控制组节点
 */
function WindowControls({
  onMinimize,
  onToggleMaximize,
  onClose,
}: Pick<TitleBarProps, "onMinimize" | "onToggleMaximize" | "onClose">) {
  return (
    <div className="flex shrink-0 items-center">
      <WindowControlButton label="Minimize" onClick={onMinimize}>
        <Minus className="size-3.5" />
      </WindowControlButton>
      <WindowControlButton label="Maximize" onClick={onToggleMaximize}>
        <Square className="size-3" />
      </WindowControlButton>
      <WindowControlButton
        label="Close"
        onClick={onClose}
        className="hover:bg-destructive/90 hover:text-destructive-foreground"
      >
        <X className="size-3.5" />
      </WindowControlButton>
    </div>
  );
}

/**
 * 可拖拽区域；双击切换最大化。
 *
 * @author Xiaoman
 * @created 2026-07-20
 *
 * @param props.className - 样式
 * @param props.children - 子节点
 * @param props.onStartDrag - 拖拽回调
 * @param props.onToggleMaximize - 双击最大化回调
 * @returns 拖拽区域节点
 */
function DragRegion({
  className,
  children,
  onStartDrag,
  onToggleMaximize,
}: {
  className?: string;
  children?: ReactNode;
  onStartDrag?: () => void;
  onToggleMaximize?: () => void;
}) {
  const handleMouseDown = (event: MouseEvent<HTMLDivElement>) => {
    if (event.button !== 0) {
      return;
    }

    if (event.detail === 2) {
      onToggleMaximize?.();
      return;
    }

    onStartDrag?.();
  };

  return (
    <div
      data-tauri-drag-region
      className={cn("cursor-default select-none", className)}
      onMouseDown={handleMouseDown}
    >
      {children}
    </div>
  );
}

/**
 * 桌面壳唯一窗口标题栏：品牌 logo + 标签 + 操作 + 窗口控制。
 *
 * @author Xiaoman
 * @created 2026-07-20
 *
 * @param props - 见 {@link TitleBarProps}
 * @returns 标题栏节点
 */
export function TitleBar({
  platform = "windows",
  tabs,
  actions,
  onStartDrag,
  onMinimize,
  onToggleMaximize,
  onClose,
  className,
  ...props
}: TitleBarProps) {
  const isMac = platform === "macos";

  const brand = (
    <img
      src="/logo.png"
      alt="OpenDesk"
      className="size-5 shrink-0 rounded-[var(--radius-sm)] object-cover"
    />
  );

  return (
    <header
      className={cn(
        "flex h-11 shrink-0 items-center overflow-hidden bg-shell",
        className,
      )}
      {...props}
    >
      {isMac ? (
        <>
          <div aria-hidden className="flex w-[4.5rem] shrink-0 items-center justify-center">
            <div className="size-3 rounded-full bg-muted" />
          </div>
          <DragRegion
            onStartDrag={onStartDrag}
            onToggleMaximize={onToggleMaximize}
            className="flex shrink-0 items-center px-3"
          >
            {brand}
          </DragRegion>
          {tabs}
          <DragRegion
            onStartDrag={onStartDrag}
            onToggleMaximize={onToggleMaximize}
            className="min-w-8 flex-1 self-stretch"
          />
          <div className="flex shrink-0 items-center gap-1 pr-2">{actions}</div>
          <div className="w-8 shrink-0" />
        </>
      ) : (
        <>
          <DragRegion
            onStartDrag={onStartDrag}
            onToggleMaximize={onToggleMaximize}
            className="flex shrink-0 items-center px-3"
          >
            {brand}
          </DragRegion>
          {tabs}
          <DragRegion
            onStartDrag={onStartDrag}
            onToggleMaximize={onToggleMaximize}
            className="min-w-8 flex-1 self-stretch"
          />
          {actions ? <div className="flex shrink-0 items-center gap-1 px-1">{actions}</div> : null}
          <WindowControls
            onMinimize={onMinimize}
            onToggleMaximize={onToggleMaximize}
            onClose={onClose}
          />
        </>
      )}
    </header>
  );
}
