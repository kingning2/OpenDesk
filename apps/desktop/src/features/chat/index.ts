import { MessageSquare } from "@desk/ui/icons";

export const chatFeature = {
  id: "chat",
  path: "/features/chat",
  navItem: {
    id: "chat",
    path: "/features/chat",
    labelKey: "nav.chat",
    icon: MessageSquare,
  },
};

export { ChatPage } from "./chat-page";
