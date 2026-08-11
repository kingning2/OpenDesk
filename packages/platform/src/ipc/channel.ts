import { invoke } from "@tauri-apps/api/core";
import type {
  ChannelIpcCloseSiteResponse,
  ChannelIpcConnectResponse,
  ChannelIpcDisconnectResponse,
  ChannelIpcLoginRequest,
  ChannelIpcLoginResponse,
  ChannelIpcOpenSiteRequest,
  ChannelIpcOpenSiteResponse,
  ChannelIpcQrCancelRequest,
  ChannelIpcQrCancelResponse,
  ChannelIpcQrCheckRequest,
  ChannelIpcQrCheckResponse,
  ChannelIpcQrStartRequest,
  ChannelIpcQrStartResponse,
  ChannelIpcSendRequest,
  ChannelIpcSendResponse,
  ChannelIpcStateRequest,
  ChannelIpcStateResponse,
} from "@desk/contracts";

export function channelStateGet(): Promise<ChannelIpcStateResponse> {
  return invoke<ChannelIpcStateResponse>("channel_state_get");
}

export function channelStateSet(
  request: ChannelIpcStateRequest,
): Promise<ChannelIpcStateResponse> {
  return invoke<ChannelIpcStateResponse>("channel_state_set", { request });
}

export function channelConnect(
  accountId: string,
): Promise<ChannelIpcConnectResponse> {
  return invoke<ChannelIpcConnectResponse>("channel_connect", { accountId });
}

export function channelDisconnect(
  accountId: string,
): Promise<ChannelIpcDisconnectResponse> {
  return invoke<ChannelIpcDisconnectResponse>("channel_disconnect", { accountId });
}

export function channelSend(
  request: ChannelIpcSendRequest,
): Promise<ChannelIpcSendResponse> {
  return invoke<ChannelIpcSendResponse>("channel_send", { request });
}

export function channelLogin(
  request: ChannelIpcLoginRequest,
): Promise<ChannelIpcLoginResponse> {
  return invoke<ChannelIpcLoginResponse>("channel_login", { request });
}

export function channelOpenSite(
  request: ChannelIpcOpenSiteRequest,
): Promise<ChannelIpcOpenSiteResponse> {
  return invoke<ChannelIpcOpenSiteResponse>("channel_open_site", { request });
}

export function channelCloseSite(): Promise<ChannelIpcCloseSiteResponse> {
  return invoke<ChannelIpcCloseSiteResponse>("channel_close_site");
}

export function channelQrStart(
  request: ChannelIpcQrStartRequest,
): Promise<ChannelIpcQrStartResponse> {
  return invoke<ChannelIpcQrStartResponse>("channel_qr_start", { request });
}

export function channelQrCheck(
  request: ChannelIpcQrCheckRequest,
): Promise<ChannelIpcQrCheckResponse> {
  return invoke<ChannelIpcQrCheckResponse>("channel_qr_check", { request });
}

export function channelQrCancel(
  request: ChannelIpcQrCancelRequest,
): Promise<ChannelIpcQrCancelResponse> {
  return invoke<ChannelIpcQrCancelResponse>("channel_qr_cancel", { request });
}
