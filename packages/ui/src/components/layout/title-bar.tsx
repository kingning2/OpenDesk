import { Minus, Square, X } from "../../icons";
import * as React from "react";

import { cn } from "../../lib/cn";

export type TitleBarPlatform = "macos" | "windows" | "linux";

export interface TitleBarProps extends React.HTMLAttributes<HTMLDivElement> {
  title?: string;
  platform?: TitleBarPlatform;
  tabs?: React.ReactNode;
  actions?: React.ReactNode;
  onStartDrag?: () => void;
  onMinimize?: () => void;
  onToggleMaximize?: () => void;
  onClose?: () => void;
}

function WindowControlButton({
  label,
  onClick,
  children,
  className,
}: {
  label: string;
  onClick?: () => void;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      onClick={onClick}
      className={cn(
        "inline-flex h-8 w-9 cursor-pointer items-center justify-center text-muted-foreground transition-colors hover:bg-muted/60 hover:text-foreground",
        className,
      )}
    >
      {children}
    </button>
  );
}

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

function DragRegion({
  className,
  children,
  onStartDrag,
  onToggleMaximize,
}: {
  className?: string;
  children?: React.ReactNode;
  onStartDrag?: () => void;
  onToggleMaximize?: () => void;
}) {
  const handleMouseDown = (event: React.MouseEvent<HTMLDivElement>) => {
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
      className={cn("select-none cursor-default", className)}
      onMouseDown={handleMouseDown}
    >
      {children}
    </div>
  );
}

export function TitleBar({
  title = "OpenDesk",
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

  return (
    <header
      className={cn(
        "flex h-11 shrink-0 items-stretch overflow-hidden bg-shell items-center",
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
            <span className="truncate font-display text-[length:var(--text-xs)] text-muted-foreground">
              {title}
            </span>
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
            <span className="font-display text-[length:var(--text-sm)] font-medium">{title}</span>
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
