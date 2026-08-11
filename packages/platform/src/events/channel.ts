/**
 * 渠道事件监听 — 入站/出站消息与连接状态推送。
 */

import type { ChannelEventMessage, ChannelEventStatus } from "@desk/contracts";
import { listenEvent } from "./index";

/** 渠道消息事件主题（与 Rust `channel.message` 对齐）。 */
export const CHANNEL_MESSAGE_EVENT = "channel.message";
/** 渠道连接状态事件主题。 */
export const CHANNEL_STATUS_EVENT = "channel.status";

/** 订阅渠道消息事件；返回取消订阅函数。 */
export function listenChannelMessage(
  handler: (payload: ChannelEventMessage) => void,
): Promise<() => void> {
  return listenEvent<ChannelEventMessage>(CHANNEL_MESSAGE_EVENT, handler);
}

/** 订阅渠道连接状态事件；返回取消订阅函数。 */
export function listenChannelStatus(
  handler: (payload: ChannelEventStatus) => void,
): Promise<() => void> {
  return listenEvent<ChannelEventStatus>(CHANNEL_STATUS_EVENT, handler);
}
