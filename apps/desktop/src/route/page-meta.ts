export interface PageMeta {
  title: string;
  description?: string;
}

const pageMetaByPath: Record<string, PageMeta> = {
  "/": {
    title: "Home",
    description: "OpenDesk architecture scaffold",
  },
  "/features/agent": {
    title: "Agent",
    description: "Sidecar connectivity vertical slice",
  },
  "/features/chat": {
    title: "Chat",
    description: "Customer conversation workspace",
  },
  "/features/mail": {
    title: "Mail",
    description: "Inbound and outbound mail handling",
  },
  "/features/knowledge": {
    title: "Knowledge",
    description: "Knowledge base and retrieval",
  },
};

export function getPageMeta(pathname: string): PageMeta {
  return pageMetaByPath[pathname] ?? { title: "OpenDesk" };
}
