/**
 * 渠道平台注册表 — 编译期仅暴露当前构建平台。
 *
 * @author coisini
 * @created 2026-07-20
 */

import { MessageSquare, Store } from "@desk/ui/icons";
import type { LucideIcon } from "@desk/ui/icons";
import { getActiveChannelPlatform } from "@desk/platform/compile";

export interface ChannelPlatform {
  /** 平台类型标识（与契约 `channel.account.kind` 对齐）。 */
  kind: string;
  /** 展示名称。 */
  name: string;
  /** 一句话描述。 */
  description: string;
  /** 工作区路径段（相对 `/features/channel`）。 */
  path: string;
  /** 图标。 */
  icon: LucideIcon;
}

/** 全部平台元数据（编译期仅当前项生效）。 */
const ALL_PLATFORMS: ChannelPlatform[] = [
  {
    kind: "xianyu",
    name: "闲鱼",
    description: "闲鱼二手交易客服 — Cookie 登录、自动回复",
    path: "xianyu",
    icon: Store,
  },
  {
    kind: "xiaohongshu",
    name: "小红书",
    description: "小红书客服（协议实现待接入）",
    path: "xiaohongshu",
    icon: MessageSquare,
  },
  {
    kind: "douyin",
    name: "抖音",
    description: "抖音客服（协议实现待接入）",
    path: "douyin",
    icon: MessageSquare,
  },
];

/** 当前编译平台（单元素列表，供渠道选择页等使用）。 */
export const CHANNEL_PLATFORMS: ChannelPlatform[] = ALL_PLATFORMS.filter(
  (platform) => platform.kind === getActiveChannelPlatform(),
);

/** 按 kind 查平台；未知返回 `null`。 */
export function getChannelPlatform(kind: string): ChannelPlatform | null {
  return ALL_PLATFORMS.find((platform) => platform.kind === kind) ?? null;
}
