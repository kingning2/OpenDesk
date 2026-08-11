import type { ChannelCookie } from "./channel_cookie";

export interface ChannelIpcQrCheckResponse {
  ok: boolean;
  status: string;
  session_id?: string;
  cookies?: ChannelCookie[];
  detail?: string;
}
