import type { ChannelAccount } from "./channel_account";
import type { ChannelConversation } from "./channel_conversation";
import type { ChannelMessage } from "./channel_message";
import type { ChannelSettings } from "./channel_settings";

export interface ChannelIpcStateResponse {
  accounts: ChannelAccount[];
  conversations: ChannelConversation[];
  messages: ChannelMessage[];
  settings: ChannelSettings;
}
