import { createBrowserRouter } from "react-router";
import { AppShell } from "../app/shell";
import { HomePage } from "../app/pages/home-page";
import { AgentPage } from "@feature/agent";

export const appRouter = createBrowserRouter([
  {
    path: "/",
    element: <AppShell />,
    children: [
      { index: true, element: <HomePage /> },
      { path: "features/agent", element: <AgentPage /> },
    ],
  },
]);
