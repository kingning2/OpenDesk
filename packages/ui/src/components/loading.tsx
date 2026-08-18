/**
 * Loading — 通用加载指示（局部 / 全屏 / 按钮三态）。
 *
 * 从原前端 `components/common/Loading.tsx` 抽取为公共组件。
 */

import { Loader2 } from "lucide-react";

import { cn } from "../lib/cn";

export interface LoadingProps {
  size?: "sm" | "md" | "lg";
  fullScreen?: boolean;
  text?: string;
  className?: string;
}

const sizes = {
  sm: "size-4",
  md: "size-8",
  lg: "size-12",
} as const;

/**
 * 加载指示器。
 *
 * @author agent
 * @created 2026-08-13
 *
 * @param props - 见 {@link LoadingProps}
 * @returns 加载节点
 */
export function Loading({ size = "md", fullScreen = false, text, className }: LoadingProps) {
  const content = (
    <div className={cn("flex flex-col items-center justify-center gap-3", className)}>
      <Loader2 className={cn("animate-spin text-primary", sizes[size])} aria-hidden />
      {text ? <p className="text-[length:var(--text-sm)] font-medium text-muted-foreground">{text}</p> : null}
    </div>
  );

  if (fullScreen) {
    return (
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm">
        {content}
      </div>
    );
  }

  return content;
}

/**
 * 页面级加载占位。
 *
 * @author agent
 * @created 2026-08-13
 *
 * @returns 页面加载节点
 */
export function PageLoading() {
  return (
    <div className="flex min-h-[400px] items-center justify-center">
      <Loading size="lg" text="加载中..." />
    </div>
  );
}
