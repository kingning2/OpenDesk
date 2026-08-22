/**
 * 编译期渠道平台工具 — 与 Vite `define` / Rust `DINGDA_CHANNEL_PLATFORM` 对齐。
 *
 * 构建参数 `DINGDA_CHANNEL_PLATFORM` 在打包时被替换为字面量，
 * Rollup 可 dead-code eliminate 未选中平台的分支与动态 import。
 *
 * @author Xiaoman
 * @created 2026-08-18
 */

/** 渠道平台 id（与契约 / Rust `ChannelKind` 对齐）。 */
export type ChannelPlatformId = "xianyu" | "ali1688" | "xiaohongshu" | "douyin";

declare const __DINGDA_CHANNEL_PLATFORM__: ChannelPlatformId;
declare const __DINGDA_CHANNEL_PLATFORMS__: readonly ChannelPlatformId[];
declare const __DINGDA_HAS_XIANYU__: boolean;
declare const __DINGDA_HAS_ALI1688__: boolean;
declare const __DINGDA_APP_BRAND_TITLE__: string;

/**
 * 标题栏品牌中文名称（编译期常量）。
 *
 * 渠道构建如「叮答（闲鱼）」；聚合 / 多平台构建为「叮答」。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */
export const APP_BRAND_TITLE: string =
  typeof __DINGDA_APP_BRAND_TITLE__ !== "undefined"
    ? __DINGDA_APP_BRAND_TITLE__
    : "叮答";

/**
 * 当前构建选定的主渠道平台 id（编译期常量）。
 *
 * 主平台：优先闲鱼；否则取首个启用平台。由 Vite `define` 注入；未注入时回退 `xianyu`（仅 dev 预览容错）。
 */
export const ACTIVE_CHANNEL_PLATFORM: ChannelPlatformId =
  typeof __DINGDA_CHANNEL_PLATFORM__ !== "undefined"
    ? __DINGDA_CHANNEL_PLATFORM__
    : "xianyu";

/**
 * 当前构建启用的全部渠道平台 id（编译期常量数组）。
 *
 * 由 Vite `define` 注入；未注入时回退 `["xianyu"]`（仅 dev 预览容错）。
 */
export const ENABLED_CHANNEL_PLATFORMS: readonly ChannelPlatformId[] =
  typeof __DINGDA_CHANNEL_PLATFORMS__ !== "undefined"
    ? __DINGDA_CHANNEL_PLATFORMS__
    : (["xianyu"] as const);

/** 闲鱼是否编入本次构建。 */
export const HAS_XIANYU: boolean =
  typeof __DINGDA_HAS_XIANYU__ !== "undefined" ? __DINGDA_HAS_XIANYU__ : true;

/** 1688 是否编入本次构建。 */
export const HAS_ALI1688: boolean =
  typeof __DINGDA_HAS_ALI1688__ !== "undefined" ? __DINGDA_HAS_ALI1688__ : false;

/** 全部已知平台 id。 */
export const CHANNEL_PLATFORM_IDS: readonly ChannelPlatformId[] = [
  "xianyu",
  "ali1688",
  "xiaohongshu",
  "douyin",
] as const;

/**
 * 判断给定 id 是否编入本次构建。
 *
 * @author Xiaoman
 * @created 2026-08-22
 *
 * @param platformId - 待判断平台 id
 * @returns 已启用为 `true`
 */
export function hasChannelPlatform(platformId: ChannelPlatformId): boolean {
  return ENABLED_CHANNEL_PLATFORMS.includes(platformId);
}

/**
 * 读取当前构建选定的渠道平台 id。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @returns 编译期平台 id
 */
export function getActiveChannelPlatform(): ChannelPlatformId {
  return ACTIVE_CHANNEL_PLATFORM;
}

/**
 * 判断给定 id 是否为当前构建选定的平台。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @param platformId - 待比较平台 id
 * @returns 一致为 `true`
 */
export function isActiveChannelPlatform(platformId: ChannelPlatformId): boolean {
  return platformId === ACTIVE_CHANNEL_PLATFORM;
}

/**
 * 断言当前构建平台 — 不匹配时在开发环境抛错。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @param platformId - 期望的平台 id
 * @returns 若匹配则返回该 id（便于类型收窄）
 */
export function assertActiveChannelPlatform(
  platformId: ChannelPlatformId,
): ChannelPlatformId {
  if (!isActiveChannelPlatform(platformId)) {
    throw new Error(
      `当前构建平台为 ${ACTIVE_CHANNEL_PLATFORM}，无法使用 ${platformId} 专用模块`,
    );
  }
  return platformId;
}

/**
 * 按编译期平台选择值 — 未选中分支在打包后会被移除。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @param branches - 各平台对应返回值
 * @returns 当前平台分支的值
 */
export function platformMatch<T>(branches: Record<ChannelPlatformId, T>): T {
  return branches[ACTIVE_CHANNEL_PLATFORM];
}
