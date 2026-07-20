/**
 * 从 `locales/{route}/{zh-cn|en-us}.json` 组装 i18next resources。
 *
 * @author Xiaoman
 * @created 2026-07-20
 */

import type { LocaleNamespaces, Messages } from "@desk/i18n";

/**
 * Vite 预加载：`locales/<route>/<locale-file>.json`
 *
 * 例：`locales/crawler/zh-cn.json` → namespace=`crawler`，语言文件=`zh-cn`
 */
const localeModules = import.meta.glob<{ default: Messages }>(
  "./locales/*/*.json",
  { eager: true },
);

/** 语言代码 → JSON 文件名（不含扩展名）。 */
const LOCALE_FILE_BY_CODE = {
  "zh-CN": "zh-cn",
  "en-US": "en-us",
} as const;

/** 桌面端支持的语言。 */
export type AppLocale = keyof typeof LOCALE_FILE_BY_CODE;

/**
 * 构建某一语言的全部 namespace 资源。
 *
 * @author Xiaoman
 * @created 2026-07-20
 *
 * @param locale - 语言代码
 * @returns namespace → 文案
 */
function buildLocaleResources(locale: AppLocale): LocaleNamespaces {
  const fileTag = LOCALE_FILE_BY_CODE[locale];
  const resources: LocaleNamespaces = {};
  const suffix = `/${fileTag}.json`;

  for (const [path, mod] of Object.entries(localeModules)) {
    if (!path.endsWith(suffix)) {
      continue;
    }
    // ./locales/crawler/zh-cn.json → crawler
    const match = path.match(/\/locales\/([^/]+)\/[^/]+\.json$/);
    if (!match) {
      continue;
    }
    resources[match[1]] = mod.default;
  }

  return resources;
}

/**
 * 桌面端全部语言资源（按路由 namespace）。
 *
 * @author Xiaoman
 * @created 2026-07-20
 */
export const appLocaleResources: Record<AppLocale, LocaleNamespaces> = {
  "zh-CN": buildLocaleResources("zh-CN"),
  "en-US": buildLocaleResources("en-US"),
};
