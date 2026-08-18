/**
 * 应用路由：壳层常驻；具体 Feature 由 WorkspaceOutlet 按标签懒加载。
 *
 * 渠道相关路由为编译期静态路径（见 `@platform-routes`），无 `:platform` 动态段。
 * `/features/channel` 重定向至当前平台的会话工作台。
 *
 * @author coisini
 * @created 2026-07-21
 */

import { createBrowserRouter, Navigate } from "react-router";
import { CHANNEL_WORKBENCH_PATH } from "@desk/platform/compile";
import { routeSegments } from "@platform-routes";

import { AppShell } from "../app/shell";

export const appRouter = createBrowserRouter([
  {
    path: "/",
    element: <AppShell />,
    children: [
      { index: true },
      { path: "features/agent" },
      { path: "features/chat" },
      {
        path: "features/channel",
        element: <Navigate to={CHANNEL_WORKBENCH_PATH} replace />,
      },
      ...routeSegments,
      { path: "features/knowledge" },
    ],
  },
]);
