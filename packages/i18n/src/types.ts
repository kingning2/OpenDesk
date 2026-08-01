/**
 * 多语言文案树与相关类型。
 *
 * @author coisini
 * @created 2026-07-20
 */

/**
 * 嵌套文案树；叶节点为字符串（供 i18next resources 使用）。
 *
 * @author coisini
 * @created 2026-07-20
 */
export type TranslationTree = {
  readonly [key: string]: string | TranslationTree;
};

/**
 * 某一语言的完整文案表。
 *
 * @author coisini
 * @created 2026-07-20
 */
export type Messages = TranslationTree;

/**
 * `t()` 插值参数。
 *
 * @author coisini
 * @created 2026-07-20
 */
export type TranslateParams = Record<string, string | number | boolean | null | undefined>;

/**
 * Replace `{key}` and `{{key}}` placeholders when the key exists in `params`.
 *
 * Unknown placeholders are left literal (e.g. docs showing `{{email}}`).
 *
 * @author Xiaoman
 * @created 2026-08-01
 *
 * @param text - Template string
 * @param params - Interpolation values
 * @returns Interpolated string
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
