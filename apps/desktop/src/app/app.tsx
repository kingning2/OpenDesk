import { useCallback } from "react";
import { RouterProvider } from "react-router";
import { appRouter } from "../route";
import { LicenseLockOverlay, useLicenseGate } from "@feature/license";
import "./globals.css";

export function App() {
  const { loading, error, gateBlocks, refresh } = useLicenseGate();

  const onActivated = useCallback(() => {
    void refresh();
  }, [refresh]);

  return (
    <div className="relative h-screen w-full overflow-hidden">
      <RouterProvider router={appRouter} />

      {loading ? (
        <div className="pointer-events-none fixed inset-x-0 bottom-0 top-11 z-40 bg-background/20 backdrop-blur-[2px]" />
      ) : null}

      {error ? (
        <div className="fixed inset-x-0 bottom-0 top-11 z-50 flex items-center justify-center bg-background/40 p-6 backdrop-blur-md">
          <div className="max-w-md space-y-3 rounded-[var(--radius-lg)] border border-border bg-card p-6 text-center shadow-lg">
            <p className="text-destructive">授权状态读取失败</p>
            <p className="text-[length:var(--text-sm)] text-muted-foreground">{error}</p>
            <button
              type="button"
              onClick={() => void refresh()}
              className="rounded-[var(--radius-md)] bg-primary px-4 py-2 text-primary-foreground"
            >
              重试
            </button>
          </div>
        </div>
      ) : null}

      {!loading && !error && gateBlocks ? (
        <LicenseLockOverlay onActivated={onActivated} />
      ) : null}
    </div>
  );
}
