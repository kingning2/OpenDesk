import type { ChannelMessage } from "./channel_message";

export interface ChannelEventMessage {
  account_id: string;
  message: ChannelMessage;
  suggestion?: string;
}
