/**
 * Mail Tauri event topics and typed listeners.
 *
 * @author Xiaoman
 * @created 2026-08-01
 */

import type { UnlistenFn } from "@tauri-apps/api/event";

import { listenEvent } from "../events";

/**
 * Mail → UI Tauri event topics.
 *
 * @author Xiaoman
 * @created 2026-08-01
 */
export enum MailUiEvent {
  /** IMAP IDLE persisted new inbound messages for an account. */
  ImapSyncUpdated = "mail:imap-sync-updated",
}

/**
 * Subscribe to IMAP IDLE persistence updates.
 *
 * @author Xiaoman
 * @created 2026-08-01
 *
 * @param onUpdated - Called with the account id that received new mail
 * @returns Unlisten function
 */
export async function listenMailImapSyncUpdated(
  onUpdated: (accountId: string) => void,
): Promise<UnlistenFn> {
  return listenEvent(MailUiEvent.ImapSyncUpdated, onUpdated);
}
