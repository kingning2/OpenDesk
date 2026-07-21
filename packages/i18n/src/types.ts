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
