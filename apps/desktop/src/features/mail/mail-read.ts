/**
 * Local read/unread helpers for mail list rows.
 *
 * @author Xiaoman
 * @created 2026-08-01
 */

import type { MailMessage } from "@desk/platform";

/**
 * Whether an inbound message is still unread in the workbench.
 *
 * @author Xiaoman
 * @created 2026-08-01
 *
 * @param message - Mail list row
 * @returns True when inbound and not marked read
 */
export function isInboundUnread(message: MailMessage): boolean {
  return message.direction === "inbound" && !message.is_read;
}
