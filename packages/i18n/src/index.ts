/**
 * `@desk/i18n` — 基于 @fluent/bundle 的通用多语言底座。
 *
 * 业务文案按「路由目录 + 语言 .ftl」放在各 app；本包提供工厂与类型。
 * 后端用户提示由 Rust 按 locale 自行翻译，不经本包做 key 映射。
 *
 * @author coisini
 * @created 2026-07-20
 * @updated 2026-08-06 由 i18next 迁移至 Fluent
 */

export type { Messages, TranslationTree, TranslateParams } from "./types";
export { interpolateTranslation } from "./types";
export { createI18n, resolveNamespaceKey } from "./create-i18n";
export type {
  CreateI18nOptions,
  I18nApi,
  I18nInstance,
  LocaleNamespaces,
} from "./create-i18n";
export type { FtlByDomain, BundlesByDomain } from "./fluent";
