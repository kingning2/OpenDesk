/**
 * Mail timestamp parsing, list date grouping, and display formatting.
 *
 * 通用时间 helper 统一委托 `@desk/utils`；本文件保留邮件特有的解析/分组逻辑。
 *
 * @author coisini
 * @created 2026-08-01
 */

import type { MailMessage } from "@desk/platform";
import {
  dateKey,
  formatClockTime as formatMailListTime,
  formatDateGroupLabel as formatMailDateGroupLabel,
  formatDateTime as formatMailTime,
  parseTimestamp as parseMailTimestamp,
} from "@desk/utils";

export {
  formatMailDateGroupLabel,
  formatMailListTime,
  formatMailTime,
  parseMailTimestamp,
};

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
 * Resolve the display/sort instant for a mail row (sent → received → created).
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
 * Group messages by calendar day for inbox / sent list sections.
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
