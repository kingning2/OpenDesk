/**
 * 编译期平台模块加载 — 利用字面量分支实现 tree-shaking。
 *
 * 调用方传入各平台的静态 `import()` 工厂；打包时仅保留
 * `ACTIVE_CHANNEL_PLATFORM` 对应分支，其余平台代码不会进入产物。
 *
 * @author Xiaoman
 * @created 2026-08-18
 */

import {
  ACTIVE_CHANNEL_PLATFORM,
  type ChannelPlatformId,
} from "./channel-platform";

/** React 页面组件占位类型（避免 @desk/platform 依赖 react）。 */
export type PlatformPageComponent = (props: Record<string, unknown>) => unknown;

/** 平台模块加载器映射。 */
export type PlatformLoaderMap<T> = Partial<
  Record<ChannelPlatformId, () => Promise<T>>
>;

/**
 * 按编译期平台加载模块 — 未选中平台分支在打包后消除。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @param loaders - 各平台静态 import 工厂
 * @returns 解析后的模块；当前平台未注册 loader 时返回 `null`
 */
export async function loadActivePlatformModule<T>(
  loaders: PlatformLoaderMap<T>,
): Promise<T | null> {
  switch (ACTIVE_CHANNEL_PLATFORM) {
    case "xianyu": {
      const load = loaders.xianyu;
      return load ? load() : null;
    }
    case "xiaohongshu": {
      const load = loaders.xiaohongshu;
      return load ? load() : null;
    }
    case "douyin": {
      const load = loaders.douyin;
      return load ? load() : null;
    }
    default: {
      const _exhaustive: never = ACTIVE_CHANNEL_PLATFORM;
      return _exhaustive;
    }
  }
}

/**
 * 按编译期平台加载 React 页面组件。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @param loaders - 各平台页面组件 import 工厂
 * @returns 页面组件；无 loader 时为 `null`
 */
export async function loadActivePlatformPage(
  loaders: PlatformLoaderMap<
    PlatformPageComponent | { default: PlatformPageComponent }
  >,
): Promise<PlatformPageComponent | null> {
  const loaded = await loadActivePlatformModule(loaders);
  if (!loaded) {
    return null;
  }
  return typeof loaded === "function"
    ? loaded
    : "default" in loaded
      ? loaded.default
      : null;
}

/**
 * 编译期平台条件执行 — 替代运行时 `if (platform === 'xianyu')`。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @param platformId - 目标平台
 * @param run - 匹配时执行的回调
 * @returns 匹配平台的返回值；不匹配为 `undefined`
 */
export function whenPlatform<T>(
  platformId: ChannelPlatformId,
  run: () => T,
): T | undefined {
  if (ACTIVE_CHANNEL_PLATFORM === platformId) {
    return run();
  }
  return undefined;
}
