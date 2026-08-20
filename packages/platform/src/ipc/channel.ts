/**
 * Channel IPC 封装。
 *
 * @author Xiaoman
 * @created 2026-08-13
 */

import type {
  ChannelIpcCloseSiteResponse,
  ChannelIpcConnectResponse,
  ChannelIpcDisconnectResponse,
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

import { call } from "./invoke";

/**
 * 读取渠道状态。
 *
 * @author Xiaoman
 * @created 2026-08-13
 */
export function channelStateGet(): Promise<ChannelIpcStateResponse> {
  return call<ChannelIpcStateResponse>("channel_state_get");
}

/**
 * 写入渠道状态。
 *
 * @author Xiaoman
 * @created 2026-08-13
 *
 * @param request - 状态请求
 */
export function channelStateSet(
  request: ChannelIpcStateRequest,
): Promise<ChannelIpcStateResponse> {
  return call<ChannelIpcStateResponse>("channel_state_set", { request });
}

/**
 * 连接渠道账号。
 *
 * @author Xiaoman
 * @created 2026-08-13
 *
 * @param accountId - 账号 ID
 */
export function channelConnect(
  accountId: string,
): Promise<ChannelIpcConnectResponse> {
  return call<ChannelIpcConnectResponse>("channel_connect", { accountId });
}

/**
 * 断开渠道账号。
 *
 * @author Xiaoman
 * @created 2026-08-13
 *
 * @param accountId - 账号 ID
 */
export function channelDisconnect(
  accountId: string,
): Promise<ChannelIpcDisconnectResponse> {
  return call<ChannelIpcDisconnectResponse>("channel_disconnect", { accountId });
}

/**
 * 发送渠道消息。
 *
 * @author Xiaoman
 * @created 2026-08-13
 *
 * @param request - 发送请求
 */
export function channelSend(
  request: ChannelIpcSendRequest,
): Promise<ChannelIpcSendResponse> {
  return call<ChannelIpcSendResponse>("channel_send", { request });
}

/**
 * 拉取会话完整消息历史（写入本地并推送；返回新插入条数）。
 *
 * @author Xiaoman
 * @created 2026-08-20
 *
 * @param conversationId - 会话 id
 */
export function channelFetchHistory(conversationId: string): Promise<number> {
  return call<number>("channel_fetch_history", { conversationId });
}

/** 会话关联商品卡信息（`message.headinfo` 返回的 data 节点）。 */
export type ProductHeadInfo = Record<string, unknown>;

/**
 * 拉取会话关联商品卡信息。
 *
 * @author Xiaoman
 * @created 2026-08-20
 *
 * @param conversationId - 会话 id
 */
export function channelProductHeadinfo(conversationId: string): Promise<ProductHeadInfo> {
  return call<ProductHeadInfo>("channel_product_headinfo", { conversationId });
}

/**
 * 打开渠道站点（主窗口内嵌子 WebView）。
 *
 * @author Xiaoman
 * @created 2026-08-13
 *
 * @param request - 账号 id + 相对主窗口客户区的逻辑像素 bounds
 */
export function channelOpenSite(
  request: ChannelIpcOpenSiteRequest,
): Promise<ChannelIpcOpenSiteResponse> {
  return call<ChannelIpcOpenSiteResponse>("channel_open_site", { request });
}

/**
 * 关闭渠道站点内嵌视图。
 *
 * @author Xiaoman
 * @created 2026-08-13
 */
export function channelCloseSite(): Promise<ChannelIpcCloseSiteResponse> {
  return call<ChannelIpcCloseSiteResponse>("channel_close_site");
}

/**
 * 启动扫码登录。
 *
 * @author Xiaoman
 * @created 2026-08-13
 *
 * @param request - 扫码启动请求
 */
export function channelQrStart(
  request: ChannelIpcQrStartRequest,
): Promise<ChannelIpcQrStartResponse> {
  return call<ChannelIpcQrStartResponse>("channel_qr_start", { request });
}

/**
 * 轮询扫码状态。
 *
 * @author Xiaoman
 * @created 2026-08-13
 *
 * @param request - 扫码检查请求
 */
export function channelQrCheck(
  request: ChannelIpcQrCheckRequest,
): Promise<ChannelIpcQrCheckResponse> {
  return call<ChannelIpcQrCheckResponse>("channel_qr_check", { request });
}

/**
 * 取消扫码登录。
 *
 * @author Xiaoman
 * @created 2026-08-13
 *
 * @param request - 取消请求
 */
export function channelQrCancel(
  request: ChannelIpcQrCancelRequest,
): Promise<ChannelIpcQrCancelResponse> {
  return call<ChannelIpcQrCancelResponse>("channel_qr_cancel", { request });
}
