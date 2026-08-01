/**
 * React 渲染错误边界：捕获组件树错误并写入日志。
 *
 * @author Xiaoman
 * @created 2026-08-01
 */

import { reportFrontendError } from "@desk/platform";
import { Component, type ErrorInfo, type ReactNode } from "react";

interface Props {
  children: ReactNode;
}

interface State {
  error: Error | null;
}

/** 捕获 React 渲染错误并落盘。 */
export class AppErrorBoundary extends Component<Props, State> {
  public constructor(props: Props) {
    super(props);
    this.state = { error: null };
  }

  public static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  public override componentDidCatch(error: Error, info: ErrorInfo): void {
    reportFrontendError({
      kind: "react",
      message: error.message || error.name || "React render error",
      component: info.componentStack?.split("\n").find((line) => line.trim())?.trim(),
      stack: error.stack,
      detail: JSON.stringify({ componentStack: info.componentStack }),
    });
  }

  public override render(): ReactNode {
    if (!this.state.error) return this.props.children;
    return (
      <div className="flex h-screen items-center justify-center bg-background p-6">
        <p className="text-muted-foreground">{this.state.error.message}</p>
      </div>
    );
  }
}
