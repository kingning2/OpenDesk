/**
 * 展示格式化工具（金额 / 数量 / 枚举文案）。
 */

/**
 * 金额格式化：保留两位小数，空值返回 `-`。
 *
 * @param value 金额（数字或字符串）
 * @returns 格式化金额
 */
export function formatAmount(value?: number | string | null): string {
  if (value === undefined || value === null || value === "") return "-";
  const num = Number(value);
  if (!Number.isFinite(num)) return "-";
  return num.toFixed(2);
}

/**
 * 数量格式化：整数展示，空值返回 `-`。
 *
 * @param value 数量
 * @returns 格式化数量
 */
export function formatQuantity(value?: number | string | null): string {
  if (value === undefined || value === null || value === "") return "-";
  const num = Number(value);
  if (!Number.isFinite(num)) return "-";
  return Number.isInteger(num) ? String(num) : num.toFixed(2);
}

/**
 * 截断文本（超长加省略号）。
 *
 * @param text 原始文本
 * @param limit 最大字符数
 * @returns 截断文本
 */
export function truncateText(text: string, limit: number): string {
  if (!text) return "";
  if (text.length <= limit) return text;
  return `${text.slice(0, limit)}...`;
}
