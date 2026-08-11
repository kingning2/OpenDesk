import type { ChannelCookie } from "./channel_cookie";

export interface ChannelIpcLoginResponse {
  ok: boolean;
  state: string;
  cookies?: ChannelCookie[];
  detail?: string;
}
