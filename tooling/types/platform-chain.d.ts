/**
 * Vite 平台链 virtual module 类型 — 供 `tsc` 使用；实际内容由 `platform-chain-plugin` 生成。
 */

declare module "virtual:dingda/platform-route-steps" {
  import type { PlatformRouteContribution } from "../../apps/desktop/src/route/platforms/types";
  export const PLATFORM_ROUTE_STEPS: PlatformRouteContribution[];
}

declare module "virtual:dingda/platform-settings-steps" {
  import type { SettingsSectionDef } from "../../apps/desktop/src/features/setting/types";
  export const PLATFORM_SETTINGS_STEPS: SettingsSectionDef[][];
}

declare module "virtual:dingda/platform-manage-nav" {
  export const MANAGE_VIEW_TITLES: Record<string, string>;
  export function isManageView(value: string): boolean;
  export type ManageView = string;
}

declare module "virtual:dingda/platform-shell-lifecycles" {
  export function PlatformShellLifecycles(): null;
}

declare module "virtual:dingda/platform-ipc-chain" {
  export * from "../../packages/platform/src/ipc/shared";
  export * from "../../packages/platform/src/ipc/platforms/xianyu";
  export * from "../../packages/platform/src/ipc/platforms/ali1688";
}
