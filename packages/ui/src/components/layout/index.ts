/**
 * 可复用页面布局原语导出。
 *
 * @author coisini
 * @created 2026-07-20
 */

export { PageContainer, pageContainerVariants } from "./page-container";
export type { PageContainerProps } from "./page-container";

export { PageScaffold } from "./page-scaffold";
export type { PageHeaderMode, PageScaffoldProps } from "./page-scaffold";

export { PageHeader } from "./page-header";
export type { PageHeaderProps } from "./page-header";

export { PageCardGrid } from "./page-card-grid";
export type { PageCardGridProps } from "./page-card-grid";

export { PageGlowCard } from "./page-glow-card";
export type { PageGlowCardProps } from "./page-glow-card";

export {
  DesktopSidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupsProvider,
  SidebarHeader,
  SidebarLink,
  SidebarProvider,
  SidebarToggle,
  SIDEBAR_WIDTH_COLLAPSED,
  SIDEBAR_WIDTH_EXPANDED,
  useSidebar,
  useSidebarGroups,
} from "./sidebar";
export type {
  SidebarContextValue,
  SidebarGroupProps,
  SidebarGroupsContextValue,
  SidebarGroupsProviderProps,
  SidebarLinkProps,
  SidebarProviderProps,
} from "./sidebar";