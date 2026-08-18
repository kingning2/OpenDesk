/**
 * 应用根组件：路由与授权门禁。
 *
 * @author coisini
 * @created 2026-07-20
 */

import { useCallback } from "react";
import { RouterProvider } from "react-router";
import { Button, Toaster } from "@desk/ui";
import { appRouter } from "../route";
import {
  LicenseGateProvider,
  LicenseLockHero,
  LicenseLockOverlay,
  useLicenseGate,
} from "@feature/license";
import "./globals.css";

/**
 * 授权门禁与路由壳。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @returns 壳节点
 */
function AppChrome() {
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
            <LicenseLockHero anim="busy" caption="正在校验授权状态…" />
          </div>
        ) : null}

        {gate.error ? (
          <div className="fixed inset-x-0 bottom-0 top-11 z-50 flex items-center justify-center bg-background/40 p-6 backdrop-blur-md">
            <div className="max-w-md space-y-3 rounded-[var(--radius-lg)] border border-border bg-card p-6 text-center shadow-lg">
              <p className="text-destructive">授权状态读取失败</p>
              <p className="text-[length:var(--text-sm)] text-muted-foreground">{gate.error}</p>
              <Button onClick={() => void gate.refresh()}>重试</Button>
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
  return <AppChrome />;
}
