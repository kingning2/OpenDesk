/**
 * Mail timestamp parsing, list date grouping, and display formatting.
 *
 * @author Xiaoman
 * @created 2026-08-01
 */

import type { MailMessage } from "@desk/platform";

/** Labels for relative day buckets in grouped mail lists. */
export type MailDateGroupLabels = {
  today: string;
  yesterday: string;
};

/** One date bucket in a grouped mail list. */
export type MailMessageDateGroup = {
  key: string;
  label: string;
  items: MailMessage[];
};

/**
 * Parse mail timestamp from epoch millis or ISO-ish strings.
 *
 * @author Xiaoman
 * @created 2026-08-01
 *
 * @param raw - Raw timestamp from contract field
 * @returns Parsed date or null when unparseable
 */
export function parseMailTimestamp(raw: string | undefined): Date | null {
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

/**
 * Resolve the display/sort instant for a mail row (sent → received → created).
 *
 * @author Xiaoman
 * @created 2026-08-01
 *
 * @param message - Mail message row
 * @returns Best-effort timestamp for grouping and list time
 */
export function messageDisplayTime(message: MailMessage): Date {
  return (
    parseMailTimestamp(message.sent_at) ??
    parseMailTimestamp(message.received_at) ??
    parseMailTimestamp(message.created_at) ??
    new Date(0)
  );
}

/**
 * Format mail timestamp millis / ISO-ish strings for detail views.
 *
 * @author Xiaoman
 * @created 2026-07-22
 *
 * @param raw - Raw timestamp string
 * @returns Localized date-time string
 */
export function formatMailTime(raw: string): string {
  const date = parseMailTimestamp(raw);
  if (!date) {
    return raw;
  }
  return date.toLocaleString();
}

/**
 * Format list-row clock time (HH:mm) under a date group header.
 *
 * @author Xiaoman
 * @created 2026-08-01
 *
 * @param date - Message instant
 * @param locale - Optional locale for `toLocaleTimeString`
 * @returns Short time label
 */
export function formatMailListTime(date: Date, locale?: string): string {
  return date.toLocaleTimeString(locale, { hour: "2-digit", minute: "2-digit" });
}

/**
 * Midnight local time for calendar-day comparisons.
 *
 * @author Xiaoman
 * @created 2026-08-01
 *
 * @param date - Any instant on the target calendar day
 * @returns Copy at 00:00:00.000 local
 */
function startOfDay(date: Date): Date {
  const copy = new Date(date);
  copy.setHours(0, 0, 0, 0);
  return copy;
}

/**
 * Stable `YYYY-MM-DD` key for grouping messages by calendar day.
 *
 * @author Xiaoman
 * @created 2026-08-01
 *
 * @param date - Calendar day source
 * @returns Sortable date key
 */
function dateKey(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

/**
 * Human label for a mail list date group (today / yesterday / full date).
 *
 * @author Xiaoman
 * @created 2026-08-01
 *
 * @param date - Group calendar day
 * @param labels - Relative day labels from i18n
 * @param locale - Optional locale for absolute dates
 * @returns Section header text
 */
export function formatMailDateGroupLabel(
  date: Date,
  labels: MailDateGroupLabels,
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

/**
 * Group messages by calendar day for inbox / sent list sections.
 *
 * @author Xiaoman
 * @created 2026-08-01
 *
 * @param messages - Messages in any order (sorted by display time desc internally)
 * @param labels - Relative day labels from i18n
 * @param locale - Optional locale for absolute date headers
 * @returns Ordered groups with section labels
 */
export function groupMessagesByDate(
  messages: MailMessage[],
  labels: MailDateGroupLabels,
  locale?: string,
): MailMessageDateGroup[] {
  const sorted = [...messages].sort(
    (left, right) => messageDisplayTime(right).getTime() - messageDisplayTime(left).getTime(),
  );

  const groups: MailMessageDateGroup[] = [];
  let currentKey: string | null = null;
  let currentLabel = "";
  let currentItems: MailMessage[] = [];

  for (const message of sorted) {
    const instant = messageDisplayTime(message);
    const key = dateKey(instant);
    const label = formatMailDateGroupLabel(instant, labels, locale);

    if (key !== currentKey) {
      if (currentKey !== null) {
        groups.push({ key: currentKey, label: currentLabel, items: currentItems });
      }
      currentKey = key;
      currentLabel = label;
      currentItems = [message];
    } else {
      currentItems.push(message);
    }
  }

  if (currentKey !== null) {
    groups.push({ key: currentKey, label: currentLabel, items: currentItems });
  }

  return groups;
}
