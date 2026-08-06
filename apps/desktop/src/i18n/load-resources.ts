/**
 * 从 `locales/{route}/{zh-CN|en-US}.ftl` 组装 Fluent bundle 资源。
 *
 * @author coisini
 * @created 2026-07-20
 * @updated 2026-08-06 由 JSON 迁移至 FTL
 */

import type { FtlByDomain } from "@desk/i18n";

/**
 * Vite 预加载：`locales/<route>/<locale-file>.ftl`
 *
 * 例：`locales/crawler/zh-CN.ftl` → namespace=`crawler`，语言文件=`zh-CN`
 */
const ftlModules = import.meta.glob(
  "./locales/*/*.ftl",
  { eager: true, query: "?raw", import: "default" },
) as Record<string, string>;

/** 语言代码 → 文件名（不含扩展名）。 */
const LOCALE_FILE_BY_CODE = {
  "zh-CN": "zh-CN",
  "en-US": "en-US",
} as const;

/** 桌面端支持的语言。 */
export type AppLocale = keyof typeof LOCALE_FILE_BY_CODE;

/**
 * 构建某一语言的全部 namespace FTL 源码。
 *
 * @author coisini
 * @created 2026-08-06
 *
 * @param locale - 语言代码
 * @returns namespace → FTL 源码
 */
function buildLocaleResources(locale: AppLocale): FtlByDomain {
  const fileTag = LOCALE_FILE_BY_CODE[locale];
  const resources: FtlByDomain = {};
  const suffix = `/${fileTag}.ftl`;

  for (const [path, source] of Object.entries(ftlModules)) {
    if (!path.endsWith(suffix)) {
      continue;
    }
    // ./locales/crawler/zh-CN.ftl → crawler
    const match = path.match(/\/locales\/([^/]+)\/[^/]+\.ftl$/);
    if (!match) {
      continue;
    }
    resources[match[1]] = source;
  }

  return resources;
}

/**
 * 桌面端全部语言资源（按路由 namespace）。
 *
 * @author coisini
 * @created 2026-07-20
 * @updated 2026-08-06
 */
export const appLocaleResources: Record<AppLocale, FtlByDomain> = {
  "zh-CN": buildLocaleResources("zh-CN"),
  "en-US": buildLocaleResources("en-US"),
};
