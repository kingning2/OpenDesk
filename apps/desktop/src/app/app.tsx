import { useCallback } from "react";
import { RouterProvider } from "react-router";
import { Toaster } from "@desk/ui";
import { appRouter } from "../route";
import {
  LicenseGateProvider,
  LicenseLockHero,
  LicenseLockOverlay,
  useLicenseGate,
} from "@feature/license";
import "./globals.css";

export function App() {
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
              <button
                type="button"
                onClick={() => void gate.refresh()}
                className="rounded-[var(--radius-md)] bg-primary px-4 py-2 text-primary-foreground"
              >
                重试
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
