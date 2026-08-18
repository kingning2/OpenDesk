/**
 * 闲鱼管理子页面导航数据 — 供管理控制台、侧栏直达项、首页入口共用。
 *
 * 抽离原因：首页 / 侧栏需要按 key 渲染入口卡片与直达链接，
 * 若从 `manage-console.tsx` 导入会连带打包全部 22 个业务页面（破坏懒加载）。
 *
 * @author agent
 * @created 2026-08-13
 */

import type { LucideIcon } from "@desk/ui/icons";
import {
  AlertTriangle,
  Ban,
  Bell,
  BookOpen,
  Filter,
  Image,
  Info,
  Layers,
  LayoutDashboard,
  MapPin,
  MessageCircle,
  MessageSquare,
  MessageSquarePlus,
  Package,
  ScrollText,
  Send,
  Shield,
  ShoppingCart,
  Ticket,
  UserCog,
  Users,
} from "@desk/ui/icons";

/** 管理子页面标识（URL 片段，如 `/manage/accounts`）。 */
export type ManageView =
  | "dashboard"
  | "accounts"
  | "keywords"
  | "items"
  | "orders"
  | "cards"
  | "blacklist"
  | "filters"
  | "channels"
  | "notifications"
  | "logs"
  | "risk"
  | "materials"
  | "publish"
  | "batch-publish"
  | "publish-logs"
  | "addresses"
  | "settings"
  | "tutorial"
  | "about"
  | "disclaimer"
  | "feedback";

/** 管理子页面导航项。 */
export interface ManageNavItem {
  key: ManageView;
  label: string;
  icon: LucideIcon;
  description: string;
}

/** 全部管理子页面（22 项，对齐原前端 Sidebar）。 */
export const MANAGE_NAV: ManageNavItem[] = [
  { key: "dashboard", label: "仪表盘", icon: LayoutDashboard, description: "业务数据总览与统计卡片" },
  { key: "accounts", label: "账号管理", icon: Users, description: "扫码登录、连接控制与多账号状态" },
  { key: "keywords", label: "自动回复", icon: MessageSquare, description: "关键词规则与自动回复配置" },
  { key: "items", label: "商品管理", icon: Package, description: "商品列表、筛选与 AI 提示词" },
  { key: "orders", label: "订单管理", icon: ShoppingCart, description: "订单列表、筛选与状态更新" },
  { key: "cards", label: "卡券管理", icon: Ticket, description: "自动发货内容库（优惠券等）" },
  { key: "blacklist", label: "黑名单", icon: Ban, description: "禁止发货买家管理" },
  { key: "filters", label: "消息过滤", icon: Filter, description: "命中关键词时跳过自动回复 / 通知" },
  { key: "channels", label: "通知渠道", icon: Bell, description: "钉钉 / 飞书 / 邮件等通知方式" },
  { key: "notifications", label: "消息通知", icon: MessageCircle, description: "账号 × 渠道绑定规则" },
  { key: "logs", label: "消息日志", icon: ScrollText, description: "自动回复成功明细" },
  { key: "risk", label: "风控日志", icon: Shield, description: "滑块验证与风控事件记录" },
  { key: "materials", label: "素材库", icon: Image, description: "发布素材，供单品 / 批量发布引用" },
  { key: "publish", label: "单品发布", icon: Send, description: "填写商品信息并发布" },
  { key: "batch-publish", label: "批量发布", icon: Layers, description: "多账号多素材并发发布" },
  { key: "addresses", label: "地址库", icon: MapPin, description: "随机地址池与个人地址" },
  { key: "publish-logs", label: "发布日志", icon: ScrollText, description: "查看所有商品发布记录及结果" },
  { key: "settings", label: "个人设置", icon: UserCog, description: "账户业务偏好配置" },
  { key: "tutorial", label: "使用教程", icon: BookOpen, description: "操作指引与常见问题" },
  { key: "about", label: "关于", icon: Info, description: "关于闲鱼自动化管理系统" },
  { key: "disclaimer", label: "免责声明", icon: AlertTriangle, description: "免责声明与使用条款" },
  { key: "feedback", label: "意见反馈", icon: MessageSquarePlus, description: "提交需求 / BUG / 建议" },
];

/** 侧栏直达的管理子页面（用户选定：仪表盘 / 账号 / 商品 / 订单）。 */
export const SIDEBAR_MANAGE_VIEWS: ManageView[] = [
  "dashboard",
  "accounts",
  "items",
  "orders",
];

/** 首页入口（其余子页面）：全部减去侧栏直达项。 */
export const HOME_MANAGE_NAV: ManageNavItem[] = MANAGE_NAV.filter(
  (item) => !SIDEBAR_MANAGE_VIEWS.includes(item.key),
);

/** 管理子页面 URL 片段 → 中文标题（标签页 / 面包屑）。 */
export const MANAGE_VIEW_TITLES: Record<ManageView, string> = Object.fromEntries(
  MANAGE_NAV.map((item) => [item.key, item.label]),
) as Record<ManageView, string>;

/** 判断字符串是否为合法管理子页面 key。 */
export function isManageView(value: string): value is ManageView {
  return value in MANAGE_VIEW_TITLES;
}

/** 构造管理子页面完整路径（编译期静态，无 platform 参数）。 */
export { managePath } from "@desk/platform/compile";
