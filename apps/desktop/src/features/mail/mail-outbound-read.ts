/**
 * Recipient open status for outbound (sent) messages.
 *
 * @author Xiaoman
 * @created 2026-08-01
 */

import type { MailMessage } from "@desk/platform";

export type OutboundRecipientReadState = "tracking_off" | "unread" | "read";

/**
 * Resolve whether the recipient has opened a tracked outbound message.
 *
 * @author Xiaoman
 * @created 2026-08-01
 *
 * @param message - Outbound mail row
 * @returns Tracking-off, unread, or read bucket
 */
export function outboundRecipientReadState(message: MailMessage): OutboundRecipientReadState {
  if (!message.open_tracking_id?.trim()) {
    return "tracking_off";
  }
  if ((message.open_count ?? 0) > 0) {
    return "read";
  }
  return "unread";
}

/**
 * i18n key for recipient read label in sent mailbox.
 *
 * @author Xiaoman
 * @created 2026-08-01
 *
 * @param message - Outbound mail row
 * @returns Key under `mail.list.*`
 */
export function outboundRecipientReadLabelKey(message: MailMessage): string {
  const state = outboundRecipientReadState(message);
  if (state === "read") {
    return "mail.list.recipientRead";
  }
  if (state === "unread") {
    return "mail.list.recipientUnread";
  }
  return "mail.list.trackingOff";
}

/**
 * Whether outbound row should use emphasis (recipient has not opened yet).
 *
 * @author Xiaoman
 * @created 2026-08-01
 */
export function isOutboundAwaitingRead(message: MailMessage): boolean {
  return outboundRecipientReadState(message) === "unread";
}
