/**
 * 异步操作按钮 — 改编自 Aceternity Stateful Button，接入设计系统 Button variant。
 *
 * @see https://ui.aceternity.com/components/stateful-button
 *
 * @author Xiaoman
 * @created 2026-08-20
 */

import * as React from "react";
import { useState } from "react";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { Check, Loader2 } from "lucide-react";
import { type VariantProps } from "class-variance-authority";

import { cn } from "../lib/cn";
import { Button, buttonVariants } from "./button";

type ButtonVariant = VariantProps<typeof buttonVariants>;

/**
 * 异步按钮属性。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export interface AsyncButtonProps
  extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, "onClick">,
    ButtonVariant {
  /** 点击回调；返回 Promise 时自动展示 loading / success。 */
  onClick?: (event: React.MouseEvent<HTMLButtonElement>) => void | Promise<void>;
  /** 外部 loading，与内部状态合并。 */
  loading?: boolean;
  /** success 状态展示时长（ms）。 */
  successDuration?: number;
}

type Phase = "idle" | "loading" | "success";

/**
 * 带 loading / success 反馈的异步按钮。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export function AsyncButton({
  className,
  variant,
  size,
  children,
  onClick,
  loading = false,
  successDuration = 1600,
  disabled,
  ...props
}: AsyncButtonProps) {
  const reducedMotion = useReducedMotion();
  const [phase, setPhase] = useState<Phase>("idle");
  const busy = loading || phase === "loading";
  const showSuccess = phase === "success" && !reducedMotion;

  async function handleClick(event: React.MouseEvent<HTMLButtonElement>) {
    if (busy || disabled) {
      return;
    }

    setPhase("loading");
    try {
      await onClick?.(event);
      if (reducedMotion) {
        setPhase("idle");
        return;
      }
      setPhase("success");
      window.setTimeout(() => setPhase("idle"), successDuration);
    } catch {
      setPhase("idle");
    }
  }

  return (
    <Button
      className={cn("relative min-w-[5.5rem]", className)}
      variant={variant}
      size={size}
      disabled={disabled || busy}
      onClick={(event) => void handleClick(event)}
      {...props}
    >
      <span className="relative flex items-center justify-center gap-2">
        <AnimatePresence mode="wait" initial={false}>
          {busy ? (
            <motion.span
              key="loading"
              initial={{ opacity: 0, scale: 0.95 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.95 }}
              transition={{ duration: 0.15 }}
              className="flex items-center gap-2"
            >
              <Loader2 className="size-3.5 animate-spin" aria-hidden />
              {children}
            </motion.span>
          ) : showSuccess ? (
            <motion.span
              key="success"
              initial={{ opacity: 0, scale: 0.95 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.95 }}
              transition={{ duration: 0.15 }}
              className="flex items-center gap-2"
            >
              <Check className="size-3.5" aria-hidden />
              {children}
            </motion.span>
          ) : (
            <motion.span
              key="idle"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.12 }}
            >
              {children}
            </motion.span>
          )}
        </AnimatePresence>
      </span>
    </Button>
  );
}
