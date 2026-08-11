import type { ChannelAccount } from "./channel_account";
import type { ChannelSettings } from "./channel_settings";

export interface ChannelIpcStateRequest {
  accounts: ChannelAccount[];
  settings: ChannelSettings;
}
