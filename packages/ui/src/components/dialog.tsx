/**
 * Dialog（模态弹窗）原语 — Radix Dialog + Motion 进出场。
 *
 * 负责：
 * - Overlay / Content / Header / Title / Description / Close
 * - 仅动画 opacity + transform（完整 transform 字符串，GPU 友好）
 * - enter ~220ms / exit ~160ms；`prefers-reduced-motion` 时去掉缩放
 *
 * @author coisini
 * @created 2026-07-21
 */

import * as DialogPrimitive from "@radix-ui/react-dialog";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import * as React from "react";

import { cn } from "../lib/cn";
import { IconButton } from "./icon-button";
import { X } from "../icons";

/** Emil 强 ease-out。 */
const EASE_OUT = [0.23, 1, 0.32, 1] as const;

const DialogOpenContext = React.createContext(false);

/**
 * Dialog 根（受控 / 非受控），并向 Content 提供 open 供 AnimatePresence 使用。
 *
 * @author coisini
 * @created 2026-07-21
 *
 * @param props - Radix Root props
 * @returns Dialog 根节点
 */
function Dialog({
  open: openProp,
  defaultOpen = false,
  onOpenChange,
  children,
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Root>) {
  const [uncontrolledOpen, setUncontrolledOpen] = React.useState(defaultOpen);
  const isControlled = openProp !== undefined;
  const open = isControlled ? Boolean(openProp) : uncontrolledOpen;

  const handleOpenChange = React.useCallback(
    (next: boolean) => {
      if (!isControlled) {
        setUncontrolledOpen(next);
      }
      onOpenChange?.(next);
    },
    [isControlled, onOpenChange],
  );

  return (
    <DialogPrimitive.Root open={open} onOpenChange={handleOpenChange} {...props}>
      <DialogOpenContext.Provider value={open}>{children}</DialogOpenContext.Provider>
    </DialogPrimitive.Root>
  );
}

/** Dialog 触发器。 */
const DialogTrigger = DialogPrimitive.Trigger;

/** Dialog 关闭。 */
const DialogClose = DialogPrimitive.Close;

/** Dialog Portal。 */
const DialogPortal = DialogPrimitive.Portal;

/**
 * Dialog 内容面板（居中模态，Motion 缩放 + 淡入）。
 *
 * Portal 常挂载；`AnimatePresence` 包在 Portal 内，保证退场可播完。
 *
 * @author coisini
 * @created 2026-07-21
 *
 * @param props - Radix Content props；可选 `showClose` / `closeLabel`
 * @returns 内容节点
 */
function DialogContent({
  className,
  children,
  showClose = true,
  closeLabel = "Close",
  dismissOnOutsidePress = true,
  onPointerDownOutside,
  onInteractOutside,
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Content> & {
  /** 是否显示右上角关闭按钮。默认 true。 */
  showClose?: boolean;
  /** 关闭按钮无障碍标签。 */
  closeLabel?: string;
  /** 点击蒙层是否关闭。默认 true。 */
  dismissOnOutsidePress?: boolean;
}) {
  const open = React.useContext(DialogOpenContext);
  const reduceMotion = useReducedMotion();

  const enterMs = reduceMotion ? 0.15 : 0.22;
  const exitMs = reduceMotion ? 0.12 : 0.16;
  const enterScale = reduceMotion ? "scale(1)" : "scale(0.96)";
  const exitScale = reduceMotion ? "scale(1)" : "scale(0.96)";

  return (
    <DialogPortal>
      <AnimatePresence>
        {open ? (
          <motion.div
            key="dialog-layer"
            className="fixed inset-0 z-[100] flex items-center justify-center p-3"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0, transition: { duration: exitMs, ease: EASE_OUT } }}
            transition={{ duration: enterMs, ease: EASE_OUT }}
          >
            <DialogPrimitive.Overlay asChild forceMount>
              <div className="absolute inset-0 bg-black/45" />
            </DialogPrimitive.Overlay>

            <DialogPrimitive.Content
              asChild
              forceMount
              onPointerDownOutside={(event) => {
                if (!dismissOnOutsidePress) {
                  event.preventDefault();
                }
                onPointerDownOutside?.(event);
              }}
              onInteractOutside={(event) => {
                if (!dismissOnOutsidePress) {
                  event.preventDefault();
                }
                onInteractOutside?.(event);
              }}
              {...props}
            >
              <motion.div
                className={cn(
                  "relative z-10 grid w-full max-w-md gap-4 overflow-hidden p-6",
                  "origin-center rounded-[var(--radius-xl)] border border-border",
                  "bg-dialog text-dialog-foreground shadow-lg",
                  "outline-none",
                  className,
                )}
                initial={{ opacity: 0, transform: enterScale }}
                animate={{ opacity: 1, transform: "scale(1)" }}
                exit={{
                  opacity: 0,
                  transform: exitScale,
                  transition: { duration: exitMs, ease: EASE_OUT },
                }}
                transition={{ duration: enterMs, ease: EASE_OUT }}
              >
                {children}
                {showClose ? (
                  <DialogPrimitive.Close asChild>
                    <IconButton label={closeLabel} className="absolute right-3 top-3">
                      <X className="size-3.5" aria-hidden />
                    </IconButton>
                  </DialogPrimitive.Close>
                ) : null}
              </motion.div>
            </DialogPrimitive.Content>
          </motion.div>
        ) : null}
      </AnimatePresence>
    </DialogPortal>
  );
}

/**
 * 兼容旧导出：遮罩由 {@link DialogContent} 内部渲染。
 *
 * @author coisini
 * @created 2026-07-21
 *
 * @returns null
 */
function DialogOverlay() {
  return null;
}

/**
 * Dialog 标题区。
 *
 * @author coisini
 * @created 2026-07-21
 *
 * @param props - div props
 * @returns 标题区节点
 */
function DialogHeader({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn("flex flex-col gap-1.5 pr-8", className)} {...props} />;
}

/**
 * Dialog 页脚。
 *
 * @author coisini
 * @created 2026-07-21
 *
 * @param props - div props
 * @returns 页脚节点
 */
function DialogFooter({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn("flex flex-col-reverse gap-2 sm:flex-row sm:justify-end", className)}
      {...props}
    />
  );
}

/**
 * Dialog 标题。
 *
 * @author coisini
 * @created 2026-07-21
 *
 * @param props - Radix Title props
 * @returns 标题节点
 */
function DialogTitle({
  className,
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Title>) {
  return (
    <DialogPrimitive.Title
      className={cn(
        "font-display text-[length:var(--text-lg)] font-semibold leading-none tracking-tight",
        className,
      )}
      {...props}
    />
  );
}

/**
 * Dialog 描述。
 *
 * @author coisini
 * @created 2026-07-21
 *
 * @param props - Radix Description props
 * @returns 描述节点
 */
function DialogDescription({
  className,
  ...props
}: React.ComponentProps<typeof DialogPrimitive.Description>) {
  return (
    <DialogPrimitive.Description
      className={cn("text-[length:var(--text-sm)] text-muted-foreground", className)}
      {...props}
    />
  );
}

export {
  Dialog,
  DialogPortal,
  DialogOverlay,
  DialogTrigger,
  DialogClose,
  DialogContent,
  DialogHeader,
  DialogFooter,
  DialogTitle,
  DialogDescription,
};
