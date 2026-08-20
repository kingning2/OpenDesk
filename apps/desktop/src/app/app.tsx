/**
 * 应用根组件：Query 上下文、路由与授权状态。
 *
 * @author coisini
 * @created 2026-07-20
 */

import { RouterProvider } from "react-router";
import { QueryProvider, Toaster } from "@desk/ui";
import { appRouter } from "../route";
import { LicenseGateProvider, useLicenseGate } from "@feature/license";
import "./globals.css";

/**
 * 授权状态 Provider 与路由壳（无全屏激活遮罩）。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @returns 壳节点
 */
function AppChrome() {
  const gate = useLicenseGate();

  return (
    <LicenseGateProvider value={gate}>
      <div className="relative h-screen w-full overflow-hidden">
        <RouterProvider router={appRouter} />
        <Toaster position="top-center" richColors closeButton />
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
    <QueryProvider>
      <AppChrome />
    </QueryProvider>
  );
}
