/**
 * 多语言文案相关类型。
 *
 * @author coisini
 * @created 2026-07-20
 * @updated 2026-08-06 迁移 Fluent 后，JSON 树类型标记为 deprecated 仅作兼容
 */

/**
 * `t()` 插值参数。
 *
 * @author coisini
 * @created 2026-07-20
 */
export type TranslateParams = Record<string, string | number | boolean | null | undefined>;

/**
 * @deprecated 已迁移 Fluent，JSON 文案树不再使用。仅作兼容保留。
 */
export type TranslationTree = {
  readonly [key: string]: string | TranslationTree;
};

/**
 * @deprecated 已迁移 Fluent，JSON 文案树不再使用。仅作兼容保留。
 */
export type Messages = TranslationTree;

/**
 * Replace `{key}` and `{{key}}` placeholders when the key exists in `params`.
 *
 * @deprecated 已迁移 Fluent，插值由 FTL 引擎处理。仅作兼容保留。
 */
export function interpolateTranslation(
  text: string,
  params?: TranslateParams,
): string {
  if (!params) {
    return text;
  }

  const replace = (key: string): string => {
    if (!(key in params)) {
      return "";
    }
    const value = params[key];
    return value === undefined || value === null ? "" : String(value);
  };

  let result = text.replace(/\{\{([\w]+)\}\}/g, (match, key: string) => {
    if (!(key in params)) {
      return match;
    }
    return replace(key);
  });

  result = result.replace(/(?<!\{)\{([\w]+)\}(?!\})/g, (match, key: string) => {
    if (!(key in params)) {
      return match;
    }
    return replace(key);
  });

  return result;
}
