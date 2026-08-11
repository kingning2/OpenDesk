import type { ChannelCookie } from "./channel_cookie";

export interface ChannelSidecarLoginResponse {
  ok: boolean;
  state: string;
  cookies?: ChannelCookie[];
  detail?: string;
  trace_id?: string;
}
