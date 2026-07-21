/**
 * 应用根组件：多语言、路由与授权门禁。
 *
 * @author coisini
 * @created 2026-07-20
 */

import { useCallback } from "react";
import { RouterProvider } from "react-router";
import { Toaster } from "@desk/ui";
import { appRouter } from "../route";
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
 * 应用根组件。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @returns 根节点
 */
export function App() {
  return (
    <I18nProvider>
      <AppChrome />
    </I18nProvider>
  );
}
