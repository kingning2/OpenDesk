import { MessageSquare } from "@desk/ui/icons";

export const channelFeature = {
  id: "channel-workbench",
  path: "/features/channel/xianyu",
  navItem: {
    id: "channel-workbench",
    path: "/features/channel/xianyu",
    label: "会话工作台",
    icon: MessageSquare,
  },
};

export { ChannelPage } from "./channel-page";
export { ChannelWorkbench } from "./channel-workbench";
export { CHANNEL_PLATFORMS, getChannelPlatform } from "./platforms";
export { useChannelStore } from "./use-channel-store";
