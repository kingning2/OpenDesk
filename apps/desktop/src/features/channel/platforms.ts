/**
 * 渠道平台注册表 — 新平台在此登记。
 *
 * 每个平台有独立的 `path`（工作区路由）与展示信息。
 * 平台工作区在 `channel-workbench.tsx` 中按 `kind` 渲染同一套「配置 + 会话」布局。
 */

import { MessageSquare, Users, Store } from "@desk/ui/icons";
import type { LucideIcon } from "@desk/ui/icons";

export interface ChannelPlatform {
  /** 平台类型标识（与契约 `channel.account.kind` 对齐）。 */
  kind: string;
  /** 展示名称。 */
  name: string;
  /** 一句话描述。 */
  description: string;
  /** 工作区路径（相对 `/features/channel`）。 */
  path: string;
  /** 图标。 */
  icon: LucideIcon;
}

/** 已支持平台。接入新平台：在此追加一条即可。 */
export const CHANNEL_PLATFORMS: ChannelPlatform[] = [
  {
    kind: "xianyu",
    name: "闲鱼",
    description: "闲鱼二手交易客服 — Cookie 登录、自动回复",
    path: "xianyu",
    icon: Store,
  },
];

/** 占位平台（架构预留，尚未实现协议）。 */
export const CHANNEL_PLATFORM_COMING_SOON: ChannelPlatform[] = [
  {
    kind: "wechat",
    name: "微信",
    description: "微信个人号客服（规划中）",
    path: "wechat",
    icon: MessageSquare,
  },
  {
    kind: "whatsapp",
    name: "WhatsApp",
    description: "WhatsApp 客服（规划中）",
    path: "whatsapp",
    icon: Users,
  },
];

/** 按 kind 查平台；未知返回 `null`。 */
export function getChannelPlatform(kind: string): ChannelPlatform | null {
  return CHANNEL_PLATFORMS.find((platform) => platform.kind === kind) ?? null;
}
