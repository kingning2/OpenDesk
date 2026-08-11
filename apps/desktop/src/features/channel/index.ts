import { MessageSquare } from "@desk/ui/icons";

export const channelFeature = {
  id: "channel",
  path: "/features/channel",
  navItem: {
    id: "channel",
    path: "/features/channel",
    label: "客服",
    icon: MessageSquare,
  },
};

export { ChannelPage } from "./channel-page";
export { ChannelWorkbench } from "./channel-workbench";
export { CHANNEL_PLATFORMS, getChannelPlatform } from "./platforms";
export { useChannelStore } from "./use-channel-store";
