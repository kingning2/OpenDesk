/**
 * 闲鱼管理子页面导航数据 — 供管理控制台、侧栏直达项、首页入口共用。
 *
 * 抽离原因：首页 / 侧栏需要按 key 渲染入口卡片与直达链接，
 * 若从 `manage-console.tsx` 导入会连带打包全部业务页面（破坏懒加载）。
 *
 * `ready: false` 的项仍保留路由与页面，但侧栏不展示（未接入平台能力）。
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
  /**
   * 是否已接入可用能力。
   * `false`：侧栏隐藏（路由/页面仍可保留，便于后续打开）。
   */
  ready: boolean;
}

/** 全部管理子页面（对齐原前端 Sidebar；侧栏仅渲染 `ready` 项）。 */
export const MANAGE_NAV: ManageNavItem[] = [
  {
    key: "accounts",
    label: "账号管理",
    icon: Users,
    description: "扫码登录、连接控制与多账号状态",
    ready: true,
  },
  {
    key: "keywords",
    label: "自动回复",
    icon: MessageSquare,
    description: "关键词规则与自动回复配置",
    ready: true,
  },
  {
    key: "items",
    label: "商品管理",
    icon: Package,
    description: "商品列表、筛选与 AI 提示词",
    ready: true,
  },
  {
    key: "orders",
    label: "订单管理",
    icon: ShoppingCart,
    description: "订单列表、筛选与状态更新",
    ready: true,
  },
  {
    key: "cards",
    label: "卡券管理",
    icon: Ticket,
    description: "自动发货内容库（优惠券等）",
    ready: true,
  },
  {
    key: "blacklist",
    label: "黑名单",
    icon: Ban,
    description: "禁止发货买家管理",
    ready: true,
  },
  {
    key: "filters",
    label: "消息过滤",
    icon: Filter,
    description: "命中关键词时跳过自动回复 / 通知",
    ready: true,
  },
  {
    key: "channels",
    label: "通知渠道",
    icon: Bell,
    description: "钉钉 / 飞书 / 邮件等通知方式",
    ready: false,
  },
  {
    key: "notifications",
    label: "消息通知",
    icon: MessageCircle,
    description: "账号 × 渠道绑定规则",
    ready: false,
  },
  {
    key: "logs",
    label: "消息日志",
    icon: ScrollText,
    description: "自动回复成功明细",
    ready: true,
  },
  {
    key: "risk",
    label: "风控日志",
    icon: Shield,
    description: "滑块验证与风控事件记录",
    ready: true,
  },
  {
    key: "materials",
    label: "素材库",
    icon: Image,
    description: "发布素材，供单品 / 批量发布引用",
    ready: false,
  },
  {
    key: "publish",
    label: "单品发布",
    icon: Send,
    description: "填写商品信息并发布",
    ready: false,
  },
  {
    key: "batch-publish",
    label: "批量发布",
    icon: Layers,
    description: "多账号多素材并发发布",
    ready: false,
  },
  {
    key: "addresses",
    label: "地址库",
    icon: MapPin,
    description: "随机地址池与个人地址",
    ready: false,
  },
  {
    key: "publish-logs",
    label: "发布日志",
    icon: ScrollText,
    description: "查看所有商品发布记录及结果",
    ready: false,
  },
  {
    key: "settings",
    label: "个人设置",
    icon: UserCog,
    description: "账户业务偏好配置",
    ready: true,
  },
  {
    key: "tutorial",
    label: "使用教程",
    icon: BookOpen,
    description: "操作指引与常见问题",
    ready: true,
  },
  {
    key: "about",
    label: "关于",
    icon: Info,
    description: "关于闲鱼自动化管理系统",
    ready: true,
  },
  {
    key: "disclaimer",
    label: "免责声明",
    icon: AlertTriangle,
    description: "免责声明与使用条款",
    ready: true,
  },
  {
    key: "feedback",
    label: "意见反馈",
    icon: MessageSquarePlus,
    description: "提交需求 / BUG / 建议",
    ready: false,
  },
];

/** 侧栏分组（Grouped Sidebar 模式）。 */
export interface ManageNavGroup {
  label: string;
  icon: LucideIcon;
  keys: ManageView[];
}

/** 侧栏分组导航 — 全部管理子页按业务域归类。 */
export const MANAGE_NAV_GROUPS: ManageNavGroup[] = [
  { label: "交易", icon: ShoppingCart, keys: ["accounts", "items", "orders", "cards", "blacklist"] },
  {
    label: "消息",
    icon: MessageSquare,
    keys: ["keywords", "filters", "channels", "notifications", "logs", "risk"],
  },
  {
    label: "发布",
    icon: Send,
    keys: ["materials", "publish", "batch-publish", "addresses", "publish-logs"],
  },
  {
    label: "系统",
    icon: UserCog,
    keys: ["settings", "tutorial", "about", "disclaimer", "feedback"],
  },
];

/**
 * 解析分组内已接入的导航项（保持 MANAGE_NAV 中的顺序）。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export function manageNavItemsForGroup(group: ManageNavGroup): ManageNavItem[] {
  const byKey = new Map(MANAGE_NAV.map((item) => [item.key, item]));
  return group.keys
    .map((key) => byKey.get(key))
    .filter((item): item is ManageNavItem => item != null && item.ready);
}

/**
 * 侧栏可见分组（去掉全部子项未接入后的空组）。
 *
 * @author Xiaoman
 * @created 2026-08-21
 */
export function visibleManageNavGroups(): Array<
  ManageNavGroup & { items: ManageNavItem[] }
> {
  return MANAGE_NAV_GROUPS.map((group) => ({
    ...group,
    items: manageNavItemsForGroup(group),
  })).filter((group) => group.items.length > 0);
}

/** @deprecated 侧栏已展示全部子页，首页不再使用 */
export const SIDEBAR_MANAGE_VIEWS: ManageView[] = ["accounts", "items", "orders"];

/** @deprecated 侧栏已展示全部子页，首页不再使用 */
export const HOME_MANAGE_NAV: ManageNavItem[] = MANAGE_NAV.filter(
  (item) => !SIDEBAR_MANAGE_VIEWS.includes(item.key) && item.ready,
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
