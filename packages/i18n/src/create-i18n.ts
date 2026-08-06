/**
 * `createI18n`：基于 @fluent/bundle 的多语言工厂。
 *
 * 每个 domain（namespace）对应一个 `.ftl` → 一个 `FluentBundle`。
 * `t("crawler.status.idle")` 解析为 namespace=`crawler`、message id=`status-idle`。
 * 未知首段回落到 `defaultNS`（通常为 `common`）。
 *
 * @author coisini
 * @created 2026-07-20
 * @updated 2026-08-06 由 i18next 迁移至 Fluent
 */

import {
  createContext,
  createElement,
  useContext,
  useMemo,
  useSyncExternalStore,
  type ReactNode,
} from "react";

import { buildLocaleBundles, translate, type FtlByDomain } from "./fluent";
import type { TranslateParams } from "./types";

/**
 * 某一语言下各 namespace 的 FTL 源码表（namespace → ftl 文本）。
 *
 * @author coisini
 * @created 2026-07-20
 */
export type LocaleNamespaces = Record<string, unknown>;

/**
 * `createI18n` 配置。
 *
 * @author coisini
 * @created 2026-07-20
 *
 * @typeParam Locale - 语言代码联合类型
 */
export interface CreateI18nOptions<Locale extends string> {
  /** 默认语言。 */
  defaultLocale: Locale;
  /**
   * 各语言 → namespace → FTL 源码。
   * 推荐结构：`locales/{route}/{zh-CN|en-US}.ftl` 加载后填入。
   */
  resources: Record<Locale, FtlByDomain>;
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
 * @author coisini
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

/** 底层兼容 shim（替代原 i18next 实例；消费端未直接使用）。 */
export interface I18nClientLike {
  /** 翻译。 */
  t: (key: string, opts?: { params?: TranslateParams }) => string;
  /** 切换语言。 */
  changeLanguage: (locale: string) => void;
  /** 当前语言。 */
  language: string;
}

/**
 * `createI18n` 返回的实例。
 *
 * @author coisini
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
  /** 底层兼容实例（原为 i18next；现为轻量 shim）。 */
  i18n: I18nClientLike;
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

function persistLocale(persistKey: string | false | undefined, locale: string) {
  if (!persistKey || typeof window === "undefined") {
    return;
  }
  try {
    window.localStorage.setItem(persistKey, locale);
  } catch {
    // ignore
  }
}

/**
 * 将 `crawler.status.idle` 拆成 namespace + 剩余 key。
 *
 * @author coisini
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
 * 创建基于 Fluent 的多语言实例（按路由 namespace）。
 *
 * @author coisini
 * @created 2026-07-20
 * @updated 2026-08-06 底层由 i18next 迁移至 Fluent
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
  namespaceSet.add(defaultNS);
  const namespaces = Array.from(namespaceSet);

  // 轻量 store：locale 与 bundle 集合的可变状态 + 订阅。
  let currentLocale: Locale = initialLocale;
  let currentBundles = buildLocaleBundles(
    initialLocale,
    options.resources[initialLocale] ?? {},
  );
  const listeners = new Set<() => void>();

  function emit() {
    for (const listener of listeners) {
      listener();
    }
  }

  function subscribe(listener: () => void): () => void {
    listeners.add(listener);
    return () => {
      listeners.delete(listener);
    };
  }

  function getSnapshot(): Locale {
    return currentLocale;
  }

  function setLocale(next: Locale) {
    if (next === currentLocale) {
      return;
    }
    currentLocale = next;
    currentBundles = buildLocaleBundles(next, options.resources[next] ?? {});
    persistLocale(persistKey, next);
    emit();
  }

  function translateKey(key: string, params?: TranslateParams): string {
    return translate(currentBundles, defaultNS, key, params);
  }

  const I18nContext = createContext<I18nApi<Locale> | null>(null);

  function I18nProvider({ children }: { children: ReactNode }) {
    const locale = useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
    const value = useMemo<I18nApi<Locale>>(
      () => ({
        locale,
        setLocale,
        locales,
        localeLabels,
        t: translateKey,
      }),
       
      [locale],
    );
    return createElement(I18nContext.Provider, { value }, children);
  }

  function useI18n(): I18nApi<Locale> {
    const api = useContext(I18nContext);
    if (!api) {
      throw new Error("useI18n must be used within <I18nProvider>");
    }
    return api;
  }

  function useT(): I18nApi<Locale>["t"] {
    return useI18n().t;
  }

  const i18n: I18nClientLike = {
    t: (key, opts) => translateKey(key, opts?.params),
    changeLanguage: (locale) => setLocale(locale as Locale),
    get language() {
      return currentLocale;
    },
  };

  return {
    I18nProvider,
    useI18n,
    useT,
    i18n,
    namespaces,
  };
}
