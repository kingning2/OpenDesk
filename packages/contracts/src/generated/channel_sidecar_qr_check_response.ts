import type { ChannelCookie } from "./channel_cookie";

export interface ChannelSidecarQrCheckResponse {
  ok: boolean;
  status: string;
  session_id?: string;
  cookies?: ChannelCookie[];
  detail?: string;
  trace_id?: string;
}
