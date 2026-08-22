/**
 * 闲鱼管理子页面导航数据 — 供管理控制台、侧栏直达项、首页入口共用。
 *
 * 抽离原因：首页 / 侧栏需要按 key 渲染入口卡片与直达链接，
 * 若从 `manage-console.tsx` 导入会连带打包全部业务页面（破坏懒加载）。
 *
 * `ready: false` 的项仍保留路由与页面，但侧栏不展示（未接入平台能力）。
 *
 * 精简说明：仅保留 账号管理 / 商品管理 / 订单管理；
 * 风控日志与免责声明已迁入应用设置弹窗。
 *
 * @author agent
 * @created 2026-08-13
 */

import type { LucideIcon } from "@desk/ui/icons";
import { Building2, Package, ShoppingCart, Users } from "@desk/ui/icons";

/** 管理子页面标识（URL 片段，如 `/manage/accounts`）。 */
export type ManageView = "dashboard" | "accounts" | "accounts-1688" | "items" | "orders";

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
    description: "闲鱼 / 1688 分 Tab 扫码登录与账号状态",
    ready: true,
  },
  ...(__DINGDA_HAS_ALI1688__
    ? [
        {
          key: "accounts-1688" as const,
          label: "1688账号",
          icon: Building2,
          description: "兼容旧路由，打开账号管理并定位 1688 Tab",
          ready: false,
        },
      ]
    : []),
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
];

/** 侧栏分组（Grouped Sidebar 模式）。 */
export interface ManageNavGroup {
  label: string;
  icon: LucideIcon;
  keys: ManageView[];
}

/** 侧栏分组导航 — 保留的管理子页按业务域归类。 */
export const MANAGE_NAV_GROUPS: ManageNavGroup[] = [
  { label: "交易", icon: ShoppingCart, keys: ["accounts", "items", "orders"] },
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
