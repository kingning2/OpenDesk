/**
 * OpenDesk 桌面端多语言实例与文案入口。
 *
 * 文案目录约定：
 * ```
 * locales/
 *   {route}/
 *     zh-cn.json
 *     en-us.json
 * ```
 * 一路由一目录；目录内每种语言一个 JSON（i18next namespace = 路由名）。
 *
 * @author coisini
 * @created 2026-07-20
 */

import { createI18n } from "@desk/i18n";

import { appLocaleResources, type AppLocale } from "./load-resources";

export type { AppLocale };

/**
 * 桌面应用 i18n 实例。
 *
 * @author coisini
 * @created 2026-07-20
 */
export const appI18n = createI18n<AppLocale>({
  defaultLocale: "zh-CN",
  defaultNS: "common",
  resources: appLocaleResources,
  localeLabels: {
    "zh-CN": "简体中文",
    "en-US": "English",
  },
  /** 兼容旧版 `en` / 大小写文件名差异。 */
  localeAliases: {
    en: "en-US",
    "en-us": "en-US",
    "zh-cn": "zh-CN",
  },
  persistKey: "opendesk.locale",
});

export const { I18nProvider, useI18n, useT } = appI18n;
