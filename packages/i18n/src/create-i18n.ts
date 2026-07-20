/**
 * `createI18n`：基于 i18next / react-i18next 的多语言工厂。
 *
 * 支持按路由（namespace）拆分资源；`t("crawler.status.idle")` 会解析为
 * namespace=`crawler`、key=`status.idle`。未知首段回落到 `defaultNS`（通常为 `common`）。
 *
 * @author Xiaoman
 * @created 2026-07-20
 */

import i18n, { type i18n as I18nClient, type Resource } from "i18next";
import {
  I18nextProvider,
  initReactI18next,
  useTranslation,
} from "react-i18next";
import { createElement, useMemo, type ReactNode } from "react";

import type { Messages, TranslateParams } from "./types";

/**
 * 某一语言下各 namespace 的文案表。
 *
 * @author Xiaoman
 * @created 2026-07-20
 */
export type LocaleNamespaces = Record<string, Messages>;

/**
 * `createI18n` 配置。
 *
 * @author Xiaoman
 * @created 2026-07-20
 *
 * @typeParam Locale - 语言代码联合类型
 */
export interface CreateI18nOptions<Locale extends string> {
  /** 默认语言。 */
  defaultLocale: Locale;
  /**
   * 各语言 → namespace → 文案。
   * 推荐结构：`locales/{route}/{zh-cn|en-us}.json` 加载后填入。
   */
  resources: Record<Locale, LocaleNamespaces>;
  /** 默认 namespace（未知 key 首段时使用）。 */
  defaultNS?: string;
  /** 语言显示名。 */
  localeLabels?: Record<Locale, string>;
  /** localStorage 键；`false` 表示不持久化。 */
  persistKey?: string | false;
  /**
   * 持久化值别名（例如旧值 `en` → `en-US`）。
   */
  localeAliases?: Record<string, Locale>;
}

/**
 * `useI18n` 返回值。
 *
 * @author Xiaoman
 * @created 2026-07-20
 *
 * @typeParam Locale - 语言代码
 */
export interface I18nApi<Locale extends string> {
  /** 当前语言。 */
  locale: Locale;
  /** 切换语言。 */
  setLocale: (locale: Locale) => void;
  /** 可用语言列表。 */
  locales: readonly Locale[];
  /** 语言显示名。 */
  localeLabels: Record<Locale, string>;
  /**
   * 翻译。
   *
   * @param key - `namespace.rest` 或 defaultNS 下的点分路径
   * @param params - 可选插值
   * @returns 文案
   */
  t: (key: string, params?: TranslateParams) => string;
}

/**
 * `createI18n` 返回的实例。
 *
 * @author Xiaoman
 * @created 2026-07-20
 *
 * @typeParam Locale - 语言代码
 */
export interface I18nInstance<Locale extends string> {
  /** 包裹应用。 */
  I18nProvider: (props: { children: ReactNode }) => ReactNode;
  /** 完整 API。 */
  useI18n: () => I18nApi<Locale>;
  /** 仅 `t`。 */
  useT: () => I18nApi<Locale>["t"];
  /** 底层 i18next 实例。 */
  i18n: I18nClient;
  /** 已注册 namespace。 */
  namespaces: readonly string[];
}

function readPersistedLocale<Locale extends string>(
  persistKey: string | false | undefined,
  defaultLocale: Locale,
  locales: readonly Locale[],
  aliases: Record<string, Locale> | undefined,
): Locale {
  if (!persistKey || typeof window === "undefined") {
    return defaultLocale;
  }
  try {
    const raw = window.localStorage.getItem(persistKey);
    if (!raw) {
      return defaultLocale;
    }
    const mapped = aliases?.[raw] ?? raw;
    if ((locales as readonly string[]).includes(mapped)) {
      return mapped as Locale;
    }
  } catch {
    // ignore
  }
  return defaultLocale;
}

/**
 * 将 `crawler.status.idle` 拆成 namespace + 剩余 key。
 *
 * @author Xiaoman
 * @created 2026-07-20
 *
 * @param key - 调用方传入的点分 key
 * @param namespaces - 已注册 namespace
 * @param defaultNS - 默认 namespace
 * @returns ns 与 key
 */
export function resolveNamespaceKey(
  key: string,
  namespaces: readonly string[],
  defaultNS: string,
): { ns: string; key: string } {
  const dot = key.indexOf(".");
  if (dot <= 0) {
    return { ns: defaultNS, key };
  }
  const head = key.slice(0, dot);
  const rest = key.slice(dot + 1);
  if (namespaces.includes(head) && rest) {
    return { ns: head, key: rest };
  }
  return { ns: defaultNS, key };
}

/**
 * 创建基于 i18next 的多语言实例（按路由 namespace）。
 *
 * @author Xiaoman
 * @created 2026-07-20
 *
 * @typeParam Locale - 语言代码
 * @param options - 配置
 * @returns Provider / hooks / i18n
 */
export function createI18n<Locale extends string>(
  options: CreateI18nOptions<Locale>,
): I18nInstance<Locale> {
  const locales = Object.keys(options.resources) as Locale[];
  const localeLabels = options.localeLabels ?? ({} as Record<Locale, string>);
  const persistKey = options.persistKey ?? false;
  const defaultNS = options.defaultNS ?? "common";
  const initialLocale = readPersistedLocale(
    persistKey,
    options.defaultLocale,
    locales,
    options.localeAliases,
  );

  const namespaceSet = new Set<string>();
  for (const locale of locales) {
    for (const ns of Object.keys(options.resources[locale] ?? {})) {
      namespaceSet.add(ns);
    }
  }
  if (!namespaceSet.has(defaultNS)) {
    namespaceSet.add(defaultNS);
  }
  const namespaces = Array.from(namespaceSet);

  const i18nResources: Resource = {};
  for (const locale of locales) {
    i18nResources[locale] = options.resources[locale] as Resource[string];
  }

  const client = i18n.createInstance();
  void client.use(initReactI18next).init({
    resources: i18nResources,
    lng: initialLocale,
    fallbackLng: options.defaultLocale,
    defaultNS,
    ns: namespaces,
    interpolation: {
      escapeValue: false,
      prefix: "{",
      suffix: "}",
    },
    returnNull: false,
  });

  function persistLocale(locale: Locale) {
    if (!persistKey || typeof window === "undefined") {
      return;
    }
    try {
      window.localStorage.setItem(persistKey, locale);
    } catch {
      // ignore
    }
  }

  function translate(
    i18nT: (key: string, opts?: Record<string, unknown>) => unknown,
    key: string,
    params?: TranslateParams,
  ): string {
    const resolved = resolveNamespaceKey(key, namespaces, defaultNS);
    return String(
      i18nT(resolved.key, {
        ns: resolved.ns,
        ...(params as Record<string, unknown>),
      }),
    );
  }

  function I18nProvider({ children }: { children: ReactNode }) {
    return createElement(I18nextProvider, { i18n: client }, children);
  }

  function useI18n(): I18nApi<Locale> {
    const { t: i18nT, i18n: instance, ready } = useTranslation(namespaces);
    const locale = (instance.language || options.defaultLocale) as Locale;

    return useMemo(() => {
      const t = (key: string, params?: TranslateParams): string => {
        if (!ready) {
          return key;
        }
        return translate(i18nT, key, params);
      };

      const setLocale = (next: Locale) => {
        void instance.changeLanguage(next);
        persistLocale(next);
      };

      return {
        locale,
        setLocale,
        locales,
        localeLabels,
        t,
      };
    }, [i18nT, instance, locale, ready]);
  }

  function useT(): I18nApi<Locale>["t"] {
    return useI18n().t;
  }

  return {
    I18nProvider,
    useI18n,
    useT,
    i18n: client,
    namespaces,
  };
}
