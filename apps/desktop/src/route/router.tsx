/**
 * 应用路由：壳层常驻；具体 Feature 由 WorkspaceOutlet 按标签懒加载。
 *
 * @author coisini
 * @created 2026-07-21
 */

import { createBrowserRouter } from "react-router";
import { AppShell } from "../app/shell";

export const appRouter = createBrowserRouter([
  {
    path: "/",
    element: <AppShell />,
    children: [
      { index: true },
      { path: "features/agent" },
      { path: "features/crawler" },
      { path: "features/crawler-results" },
      { path: "features/customer" },
      { path: "features/chat" },
      { path: "features/mail" },
      { path: "features/workflow" },
      { path: "features/knowledge" },
    ],
  },
]);
