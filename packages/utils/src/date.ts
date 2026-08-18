/**
 * 日期/时间格式化工具（抽取自原 xianyu 前端 utils/date.ts）。
 */

/**
 * 将日期值格式化为「YYYY/MM/DD HH:mm:ss」中文 24 小时格式。
 * 空值 / 非法值返回 `-`。
 *
 * @param value 字符串、Date 对象或空值
 * @returns 格式化后的中文日期时间字符串，或缺省占位符 `-`
 */
export function formatDateTime(value?: string | Date | null): string {
  if (!value) return "-";
  const date = typeof value === "string" ? new Date(value) : value;
  if (Number.isNaN(date.getTime())) return "-";
  return date.toLocaleString("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

/**
 * 获取北京时间日期字符串（YYYY-MM-DD，input[type="date"] 可直接使用）。
 *
 * @param value 可选日期，默认当前时间
 * @returns 北京时间日期字符串
 */
export function getBeijingDateInputValue(value: Date = new Date()): string {
  const parts = new Intl.DateTimeFormat("zh-CN", {
    timeZone: "Asia/Shanghai",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).formatToParts(value);

  const partMap = Object.fromEntries(parts.map((part) => [part.type, part.value]));
  return `${partMap.year}-${partMap.month}-${partMap.day}`;
}

/**
 * 格式化毫秒时间戳为「MM-DD HH:mm」短格式（会话列表等紧凑场景）。
 *
 * @param timestampMs 毫秒时间戳
 * @returns 短格式时间
 */
export function formatShortTime(timestampMs: number | string): string {
  const millis = Number(timestampMs);
  if (!Number.isFinite(millis) || millis <= 0) return "-";
  const date = new Date(millis);
  if (Number.isNaN(date.getTime())) return "-";
  return date.toLocaleString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}
