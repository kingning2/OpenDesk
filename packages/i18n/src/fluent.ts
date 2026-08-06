/**
 * Fluent 核心：按 domain 构建 FluentBundle，解析 `{namespace}.{rest}` key。
 *
 * Fluent message id 扁平且不含点号，因此：
 * - 每个 domain（namespace）对应一个 `.ftl` → 一个 `FluentBundle`。
 * - `t("crawler.status.idle")` → namespace=`crawler`，message id=`status-idle`（`.` 转 `-`）。
 *
 * @author coisini
 * @created 2026-08-06
 */

import { FluentBundle, FluentResource, type FluentVariable } from "@fluent/bundle";

import type { TranslateParams } from "./types";

/** 某语言下各 domain 的 FTL 源码（domain → ftl 文本）。 */
export type FtlByDomain = Record<string, string>;

/** 某一语言的所有 domain bundle（domain → FluentBundle）。 */
export type BundlesByDomain = Record<string, FluentBundle>;

/**
 * 从 FTL 源码构建该语言的 domain bundle 集合。
 *
 * @author coisini
 * @created 2026-08-06
 *
 * @param locale - 语言代码（如 `zh-CN`）
 * @param ftlByDomain - domain → FTL 源码
 * @returns domain → FluentBundle
 */
export function buildLocaleBundles(
  locale: string,
  ftlByDomain: FtlByDomain,
): BundlesByDomain {
  const bundles: BundlesByDomain = {};
  for (const [domain, source] of Object.entries(ftlByDomain)) {
    const resource = new FluentResource(source);
    const bundle = new FluentBundle(locale, { useIsolating: false });
    bundle.addResource(resource);
    bundles[domain] = bundle;
  }
  return bundles;
}

/** 过滤掉 null/undefined 参数（FluentVariable 不接受它们）。 */
function toFluentVars(params: TranslateParams | undefined): Record<string, FluentVariable> {
  if (!params) {
    return {};
  }
  const vars: Record<string, FluentVariable> = {};
  for (const [key, value] of Object.entries(params)) {
    if (value === undefined || value === null) {
      continue;
    }
    vars[key] = value;
  }
  return vars;
}

/**
 * 将 `a.b.c` 的 rest 部分（`b.c`）转为 Fluent message id（`b-c`）。
 *
 * @author coisini
 * @created 2026-08-06
 */
function toMessageId(rest: string): string {
  return rest.split(".").join("-");
}

/**
 * 按 namespace 首段从 bundle 集合中解析并格式化文案。
 *
 * @author coisini
 * @created 2026-08-06
 *
 * @param bundles - domain → FluentBundle
 * @param defaultNS - 未知首段时回落的 namespace
 * @param key - 形如 `namespace.rest` 的点分 key
 * @param params - 插值参数
 * @returns 译文；message 缺失时返回原 key
 */
export function translate(
  bundles: BundlesByDomain,
  defaultNS: string,
  key: string,
  params?: TranslateParams,
): string {
  const dot = key.indexOf(".");
  let ns = defaultNS;
  let rest = key;
  if (dot > 0) {
    const head = key.slice(0, dot);
    const tail = key.slice(dot + 1);
    if (tail && head in bundles) {
      ns = head;
      rest = tail;
    }
  }
  const bundle = bundles[ns];
  if (!bundle) {
    return key;
  }
  const id = toMessageId(rest);
  const message = bundle.getMessage(id);
  if (!message || message.value === null) {
    return key;
  }
  const vars = toFluentVars(params);
  const errors: Error[] = [];
  const result = bundle.formatPattern(message.value, vars, errors);
  if (errors.length > 0) {
    console.warn(`[i18n] format error for "${key}":`, errors);
  }
  return result;
}
