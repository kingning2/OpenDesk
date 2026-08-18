/**
 * 消息通知 IPC 封装 — 通知渠道 CRUD + 账号×渠道绑定规则。
 *
 * 后端：壳层 `commands/notification.rs`（InMemoryNotificationStore + app::notification::NotificationService）。
 *
 * @author agent
 * @created 2026-08-13
 */

import { call } from "./invoke";

/** 通知渠道类型（与 Rust `ChannelKind` 对齐）。 */
export type ChannelKind =
  | "dingtalk"
  | "feishu"
  | "bark"
  | "email"
  | "webhook"
  | "wechat"
  | "telegram"
  | "pushplus";

/** 通知渠道（与 Rust `NotificationChannel` 对齐；config 为 JSON 字符串）。 */
export interface NotificationChannel {
  id: number;
  owner_id: number;
  name: string;
  kind: ChannelKind;
  config: string;
  enabled: boolean;
}

/** 消息通知绑定（与 Rust `MessageNotification` 对齐）。 */
export interface MessageNotification {
  id: number;
  owner_id: number;
  account_id: string;
  channel_id: number;
  enabled: boolean;
  channel_name: string | null;
}

/** 渠道类型展示配置（对齐原前端 channelTypes）。 */
export interface ChannelTypeInfo {
  type: ChannelKind;
  label: string;
  desc: string;
  placeholder: string;
  defaultConfig: Record<string, unknown>;
}

/** 全部渠道类型目录（配置弹窗默认填充样例配置）。 */
export const CHANNEL_TYPES: ChannelTypeInfo[] = [
  {
    type: "dingtalk",
    label: "钉钉通知",
    desc: "钉钉机器人消息",
    placeholder: '{"webhook_url": "https://oapi.dingtalk.com/robot/send?access_token=..."}',
    defaultConfig: {
      webhook_url: "https://oapi.dingtalk.com/robot/send?access_token=你的access_token",
      secret: "",
    },
  },
  {
    type: "feishu",
    label: "飞书通知",
    desc: "飞书机器人消息",
    placeholder: '{"webhook_url": "https://open.feishu.cn/open-apis/bot/v2/hook/..."}',
    defaultConfig: {
      webhook_url: "https://open.feishu.cn/open-apis/bot/v2/hook/你的hook_id",
    },
  },
  {
    type: "bark",
    label: "Bark通知",
    desc: "iOS推送通知",
    placeholder: '{"device_key": "xxx", "server_url": "https://api.day.app"}',
    defaultConfig: { device_key: "你的设备密钥", server_url: "https://api.day.app" },
  },
  {
    type: "email",
    label: "邮件通知",
    desc: "SMTP邮件发送",
    placeholder:
      '{"smtp_server": "...", "smtp_port": 587, "email_user": "...", "email_password": "...", "recipient_email": "..."}',
    defaultConfig: {
      smtp_server: "smtp.qq.com",
      smtp_port: 587,
      email_user: "你的邮箱@qq.com",
      email_password: "你的授权码",
      recipient_email: "接收邮箱@example.com",
    },
  },
  {
    type: "webhook",
    label: "Webhook",
    desc: "自定义HTTP请求",
    placeholder: '{"webhook_url": "https://..."}',
    defaultConfig: { webhook_url: "https://你的webhook地址", method: "POST", headers: {} },
  },
  {
    type: "wechat",
    label: "微信通知",
    desc: "企业微信机器人",
    placeholder: '{"webhook_url": "https://qyapi.weixin.qq.com/cgi-bin/webhook/send?key=..."}',
    defaultConfig: {
      webhook_url: "https://qyapi.weixin.qq.com/cgi-bin/webhook/send?key=你的key",
    },
  },
  {
    type: "telegram",
    label: "Telegram",
    desc: "Telegram机器人",
    placeholder: '{"bot_token": "...", "chat_id": "..."}',
    defaultConfig: { bot_token: "你的Bot_Token", chat_id: "你的Chat_ID" },
  },
  {
    type: "pushplus",
    label: "PushPlus",
    desc: "微信公众号推送",
    placeholder: '{"token": "...", "topic": "", "template": "txt"}',
    defaultConfig: { token: "你的token", topic: "", template: "txt" },
  },
];

/** 渠道类型中文标签查询。 */
export function channelKindLabel(kind: ChannelKind): string {
  return CHANNEL_TYPES.find((info) => info.type === kind)?.label ?? kind;
}

/** 查询通知渠道列表。 */
export function notificationChannelList(ownerId: number): Promise<NotificationChannel[]> {
  return call<NotificationChannel[]>("notification_channel_list", { ownerId });
}

/** 新建通知渠道。 */
export function notificationChannelCreate(
  ownerId: number,
  channel: Pick<NotificationChannel, "name" | "kind" | "config" | "enabled">,
): Promise<NotificationChannel> {
  return call<NotificationChannel>("notification_channel_create", {
    ownerId,
    channel: { ...channel, id: 0, owner_id: ownerId },
  });
}

/** 更新通知渠道。 */
export function notificationChannelUpdate(
  ownerId: number,
  channel: NotificationChannel,
): Promise<void> {
  return call<void>("notification_channel_update", { ownerId, channel });
}

/** 切换渠道启用状态。 */
export function notificationChannelSetEnabled(
  ownerId: number,
  channelId: number,
  enabled: boolean,
): Promise<void> {
  return call<void>("notification_channel_set_enabled", {
    request: { owner_id: ownerId, channel_id: channelId, enabled },
  });
}

/** 测试通知渠道（配置校验）。 */
export function notificationChannelTest(ownerId: number, channelId: number): Promise<string> {
  return call<string>("notification_channel_test", {
    request: { owner_id: ownerId, channel_id: channelId },
  });
}

/** 删除通知渠道。 */
export function notificationChannelDelete(ownerId: number, channelId: number): Promise<void> {
  return call<void>("notification_channel_delete", {
    request: { owner_id: ownerId, channel_id: channelId },
  });
}

/** 查询消息通知列表。 */
export function notificationList(ownerId: number): Promise<MessageNotification[]> {
  return call<MessageNotification[]>("notification_list", { ownerId });
}

/** upsert 消息通知（同账号同渠道更新，否则新建）。 */
export function notificationSet(
  ownerId: number,
  accountId: string,
  channelId: number,
  enabled: boolean,
): Promise<MessageNotification> {
  return call<MessageNotification>("notification_set", {
    request: { owner_id: ownerId, account_id: accountId, channel_id: channelId, enabled },
  });
}

/** 删除消息通知。 */
export function notificationDelete(ownerId: number, notificationId: number): Promise<void> {
  return call<void>("notification_delete", {
    request: { owner_id: ownerId, notification_id: notificationId },
  });
}
