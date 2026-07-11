import type { Messages } from "../types";

const en = {
  app: {
    name: "OpenDesk",
  },
  nav: {
    home: "Home",
    agent: "Agent",
    chat: "Chat",
    mail: "Mail",
    knowledge: "Knowledge",
  },
  settings: {
    title: "Settings",
    language: "Language",
    languageZh: "简体中文",
    languageEn: "English",
  },
  pages: {
    home: {
      title: "OpenDesk",
      description: "Architecture scaffold — select a feature from the sidebar.",
    },
    agent: {
      subtitle: "Sidecar connectivity vertical slice",
      ping: "Ping sidecar",
      pinging: "Pinging...",
    },
    chat: {
      description: "Customer session workspace — in development.",
    },
    mail: {
      description: "Inbound and outbound mail — in development.",
    },
    knowledge: {
      description: "Knowledge base and retrieval — in development.",
    },
    placeholder: {
      developing: "This feature is under development.",
    },
  },
} as const satisfies Messages;

export default en;
