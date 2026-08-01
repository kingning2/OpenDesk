/**
 * 应用根组件：多语言、路由与授权门禁。
 *
 * @author coisini
 * @created 2026-07-20
 */

import { useCallback, useEffect } from "react";
import { RouterProvider } from "react-router";
import { Toaster, toast } from "@desk/ui";
import {
  installIpcErrorReporter,
  reportFrontendError,
  reportFrontendErrorValue,
} from "@desk/platform";
import { appRouter } from "../route";
import { AppErrorBoundary } from "./app-error-boundary";
import { I18nProvider, useT } from "../i18n";
import {
  LicenseGateProvider,
  LicenseLockHero,
  LicenseLockOverlay,
  useLicenseGate,
} from "@feature/license";
import "./globals.css";

/**
 * 授权门禁与路由壳（需在 I18nProvider 内）。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @returns 壳节点
 */
function AppChrome() {
  const t = useT();
  const gate = useLicenseGate();

  const onActivated = useCallback(() => {
    void gate.refresh();
  }, [gate]);

  return (
    <LicenseGateProvider value={gate}>
      <div className="relative h-screen w-full overflow-hidden">
        <RouterProvider router={appRouter} />
        <Toaster position="top-center" richColors closeButton />

        {gate.loading ? (
          <div
            className="fixed inset-x-0 bottom-0 top-11 z-40 flex items-center justify-center bg-background/40 p-6 backdrop-blur-md"
            role="status"
            aria-live="polite"
          >
            <LicenseLockHero anim="busy" caption={t("license.checking")} />
          </div>
        ) : null}

        {gate.error ? (
          <div className="fixed inset-x-0 bottom-0 top-11 z-50 flex items-center justify-center bg-background/40 p-6 backdrop-blur-md">
            <div className="max-w-md space-y-3 rounded-[var(--radius-lg)] border border-border bg-card p-6 text-center shadow-lg">
              <p className="text-destructive">{t("license.readFailed")}</p>
              <p className="text-[length:var(--text-sm)] text-muted-foreground">{gate.error}</p>
              <button
                type="button"
                onClick={() => void gate.refresh()}
                className="rounded-[var(--radius-md)] bg-primary px-4 py-2 text-primary-foreground"
              >
                {t("license.retry")}
              </button>
            </div>
          </div>
        ) : null}

        {!gate.loading && !gate.error && gate.gateBlocks ? (
          <LicenseLockOverlay onActivated={onActivated} />
        ) : null}
      </div>
    </LicenseGateProvider>
  );
}

/**
 * 安装错误上报：IPC toast、window.onerror、未处理的 Promise 拒绝。
 *
 * @author Xiaoman
 * @created 2026-08-01
 */
function installErrorReporters(): void {
  // IPC 失败弹 toast，相同错误只弹一次
  installIpcErrorReporter((_command, message) => {
    toast.error(message);
  });

  // 全局 JS 错误
  window.addEventListener("error", (event) => {
    reportFrontendError({
      kind: "uncaught",
      message: event.message || "Uncaught error",
      source: event.filename || undefined,
      line: event.lineno || undefined,
      column: event.colno || undefined,
      stack: event.error instanceof Error ? event.error.stack : undefined,
    });
  });

  // 未捕获的 Promise 拒绝
  window.addEventListener("unhandledrejection", (event) => {
    reportFrontendErrorValue("unhandledrejection", event.reason);
  });
}

/**
 * 应用根组件。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @returns 根节点
 */
export function App() {
  useEffect(() => {
    installErrorReporters();
  }, []);

  return (
    <I18nProvider>
      <AppErrorBoundary>
        <AppChrome />
      </AppErrorBoundary>
    </I18nProvider>
  );
}
