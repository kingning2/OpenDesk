import { createBrowserRouter } from "react-router";
import { AppShell } from "../app/shell";
import { HomePage } from "../app/pages/home-page";
import { AgentPage } from "@feature/agent";
import { ChatPage } from "@feature/chat";
import { CrawlerPage } from "@feature/crawler";
import { MailPage } from "@feature/mail";
import { KnowledgePage } from "@feature/knowledge";

export const appRouter = createBrowserRouter([
  {
    path: "/",
    element: <AppShell />,
    children: [
      { index: true, element: <HomePage /> },
      { path: "features/agent", element: <AgentPage /> },
      { path: "features/crawler", element: <CrawlerPage /> },
      { path: "features/chat", element: <ChatPage /> },
      { path: "features/mail", element: <MailPage /> },
      { path: "features/knowledge", element: <KnowledgePage /> },
    ],
  },
]);
