/**
 * 通用时间工具：时间戳解析、日期分组、显示格式化。
 */

/** 相对日期分组的标签（i18n 提供）。 */
export type RelativeDayLabels = {
  today: string;
  yesterday: string;
};

/** 解析 epoch 毫秒或 ISO-ish 字符串为 Date；无法解析返回 null。 */
export function parseTimestamp(raw: string | undefined): Date | null {
  if (!raw) {
    return null;
  }
  const asNumber = Number(raw);
  if (Number.isFinite(asNumber) && asNumber > 0) {
    return new Date(asNumber);
  }
  const parsed = Date.parse(raw);
  if (Number.isFinite(parsed)) {
    return new Date(parsed);
  }
  return null;
}

/** 将时间戳字符串格式化为本地日期时间；无效则原样返回。 */
export function formatDateTime(raw: string): string {
  const date = parseTimestamp(raw);
  if (!date) {
    return raw;
  }
  return date.toLocaleString();
}

/** 本地 `HH:mm` 时钟时间。 */
export function formatClockTime(date: Date, locale?: string): string {
  return date.toLocaleTimeString(locale, { hour: "2-digit", minute: "2-digit" });
}

/** 本地当日零点（用于日历日比较）。 */
export function startOfDay(date: Date): Date {
  const copy = new Date(date);
  copy.setHours(0, 0, 0, 0);
  return copy;
}

/** 稳定 `YYYY-MM-DD` 日期分组 key。 */
export function dateKey(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

/** 日期分组展示标签（今天 / 昨天 / 完整日期）。 */
export function formatDateGroupLabel(
  date: Date,
  labels: RelativeDayLabels,
  locale?: string,
): string {
  const today = startOfDay(new Date());
  const yesterday = new Date(today);
  yesterday.setDate(yesterday.getDate() - 1);
  const day = startOfDay(date);

  if (day.getTime() === today.getTime()) {
    return labels.today;
  }
  if (day.getTime() === yesterday.getTime()) {
    return labels.yesterday;
  }
  return date.toLocaleDateString(locale, { year: "numeric", month: "long", day: "numeric" });
}

/** 将应用内 locale 映射为 BCP 47 标签（`zh-CN` / `en-US`）。 */
export function localeTag(locale: string): string {
  if (locale === "zh-CN") {
    return "zh-CN";
  }
  return "en-US";
}

/** 格式化日志时间（本地时分秒）；无效 ISO 返回空串。 */
export function formatLogTime(iso: string, locale: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) {
    return "";
  }
  return date.toLocaleTimeString(localeTag(locale), {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

/** 会话列表行时间：今天显示时分，往年显示完整日期。 */
export function formatSessionTime(timestamp: number): string {
  const date = new Date(timestamp);
  const now = new Date();
  if (date.toDateString() === now.toDateString()) {
    return new Intl.DateTimeFormat(undefined, {
      hour: "2-digit",
      minute: "2-digit",
    }).format(date);
  }
  if (date.getFullYear() === now.getFullYear()) {
    return new Intl.DateTimeFormat(undefined, {
      month: "2-digit",
      day: "2-digit",
    }).format(date);
  }
  return new Intl.DateTimeFormat(undefined, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).format(date);
}

/** 当前 Unix 秒。 */
export function unixSeconds(): number {
  return Math.floor(Date.now() / 1000);
}
