/**
 * React 错误边界：捕获子树渲染/生命周期错误并展示可诊断信息。
 *
 * 注意：无法捕获事件处理器内错误，也无法阻止「Maximum update depth」在抛出前的风暴；
 * 一旦 React 抛出该错误，本边界可展示 message + componentStack，便于定位。
 *
 * @author coisini
 * @created 2026-07-23
 */

import * as React from "react";

import { cn } from "../lib/cn";
import { Button } from "./button";

/**
 * ErrorBoundary 属性。
 *
 * @author coisini
 * @created 2026-07-23
 */
export interface ErrorBoundaryProps {
  /** 被保护的子树。 */
  children: React.ReactNode;
  /** 可选标题。 */
  title?: string;
  /** 根 className。 */
  className?: string;
  /**
   * 错误回调（可用于日志）。
   *
   * @param error - 捕获到的错误
   * @param info - React 组件栈信息
   */
  onError?: (error: Error, info: React.ErrorInfo) => void;
  /**
   * 自定义 fallback；不传则用默认诊断面板。
   *
   * @param ctx - 错误上下文
   * @returns 回退 UI
   */
  fallback?: (ctx: {
    error: Error;
    componentStack: string | null;
    reset: () => void;
  }) => React.ReactNode;
}

interface ErrorBoundaryState {
  error: Error | null;
  componentStack: string | null;
}

/**
 * 捕获子组件错误的边界。
 *
 * @author coisini
 * @created 2026-07-23
 */
export class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
  override state: ErrorBoundaryState = {
    error: null,
    componentStack: null,
  };

  /**
   * 从错误派生 UI 状态。
   *
   * @author coisini
   * @created 2026-07-23
   *
   * @param error - 抛出的错误
   * @returns 下一 state
   */
  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    return { error };
  }

  /**
   * 记录 componentStack 并通知外部。
   *
   * @author coisini
   * @created 2026-07-23
   *
   * @param error - 抛出的错误
   * @param info - React 错误信息
   */
  override componentDidCatch(error: Error, info: React.ErrorInfo): void {
    this.setState({ componentStack: info.componentStack ?? null });
    this.props.onError?.(error, info);
    console.error("[ErrorBoundary]", error, info.componentStack);
  }

  /**
   * 清除错误并尝试重新渲染子树。
   *
   * @author coisini
   * @created 2026-07-23
   */
  reset = (): void => {
    this.setState({ error: null, componentStack: null });
  };

  override render(): React.ReactNode {
    const { error, componentStack } = this.state;
    if (!error) {
      return this.props.children;
    }

    if (this.props.fallback) {
      return this.props.fallback({
        error,
        componentStack,
        reset: this.reset,
      });
    }

    return (
      <div
        className={cn(
          "flex min-h-0 flex-1 flex-col gap-3 overflow-auto rounded-[var(--radius-md)] border border-red-500/40 bg-red-500/5 p-4 text-red-700 dark:text-red-300",
          this.props.className,
        )}
        role="alert"
      >
        <div className="text-[length:var(--text-sm)] font-semibold">
          {this.props.title ?? "组件渲染失败"}
        </div>
        <pre className="whitespace-pre-wrap break-words text-[length:var(--text-xs)] opacity-90">
          {error.name}: {error.message}
        </pre>
        {componentStack ? (
          <details open className="text-[length:var(--text-xs)]">
            <summary className="cursor-pointer font-medium">componentStack</summary>
            <pre className="mt-2 whitespace-pre-wrap break-words opacity-80">{componentStack}</pre>
          </details>
        ) : null}
        <div>
          <Button type="button" size="sm" variant="outline" onClick={this.reset}>
            重试
          </Button>
        </div>
      </div>
    );
  }
}
