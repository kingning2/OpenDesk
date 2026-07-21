import type { ComponentType } from "react";
import { HomePage } from "./pages/home-page";
import { AgentPage } from "@feature/agent";
import { ChatPage } from "@feature/chat";
import { CrawlerPage } from "@feature/crawler";
import { CustomerPage } from "@feature/customer";
import { MailPage } from "@feature/mail";
import { KnowledgePage } from "@feature/knowledge";
import { cn } from "@desk/ui";

/** Static workspace routes — mounted once per open tab (keep-alive). */
const WORKSPACE_PAGES: Record<string, ComponentType> = {
  "/": HomePage,
  "/features/agent": AgentPage,
  "/features/crawler": CrawlerPage,
  "/features/customer": CustomerPage,
  "/features/chat": ChatPage,
  "/features/mail": MailPage,
  "/features/knowledge": KnowledgePage,
};

export interface WorkspaceOutletProps {
  openPaths: string[];
  activePath: string;
}

/**
 * Tab keep-alive: open workspace tabs stay mounted (hidden) so long-running
 * features (e.g. crawler polling) survive tab switches.
 */
export function WorkspaceOutlet({ openPaths, activePath }: WorkspaceOutletProps) {
  return (
    <>
      {openPaths.map((path) => {
        const Page = WORKSPACE_PAGES[path];
        if (!Page) {
          return null;
        }
        const isActive = path === activePath;
        return (
          <div
            key={path}
            className={cn(
              "min-h-0 flex-1 flex-col overflow-hidden",
              isActive ? "flex" : "hidden",
            )}
            aria-hidden={!isActive}
          >
            <Page />
          </div>
        );
      })}
    </>
  );
}
