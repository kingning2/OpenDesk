import { invokeIpc } from "./invoke";
import type {
  MailDtoImapSyncState,
  MailDtoMailAccount,
  MailDtoMailMessage,
  MailDtoMailTemplate,
  MailIpcAccountListResponse,
  MailIpcAccountSaveRequest,
  MailIpcGenerateHtmlRequest,
  MailIpcGenerateHtmlResponse,
  MailIpcInboxUnmatchedListRequest,
  MailIpcInboxUnmatchedListResponse,
  MailIpcLinkInboundCustomerRequest,
  MailIpcLinkInboundCustomerResponse,
  MailIpcMessageListRequest,
  MailIpcMessageListResponse,
  MailIpcMessageMarkReadRequest,
  MailIpcMessageMarkReadResponse,
  MailIpcRecordInboundRequest,
  MailIpcRecordInboundResponse,
  MailIpcSendRequest,
  MailIpcSendResponse,
  MailIpcSyncNowRequest,
  MailIpcSyncNowResponse,
  MailIpcSyncStatusRequest,
  MailIpcSyncStatusResponse,
  MailIpcTemplateApplyRequest,
  MailIpcTemplateApplyResponse,
  MailIpcTemplateListResponse,
  MailIpcTemplateSaveRequest,
} from "@desk/contracts";

export type MailTemplate = MailDtoMailTemplate;
export type MailAccount = MailDtoMailAccount;
export type MailMessage = MailDtoMailMessage;
export type MailImapSyncState = MailDtoImapSyncState;

/**
 * Load all mail templates.
 *
 * @author coisini
 * @created 2026-07-21
 */
export async function mailTemplateList(): Promise<{ items: MailTemplate[]; total: number }> {
  const response = await invokeIpc<MailIpcTemplateListResponse>("mail_template_list");
  try {
    const parsed = JSON.parse(response.templates_json ?? "[]") as MailTemplate[];
    return { items: Array.isArray(parsed) ? parsed : [], total: response.total ?? 0 };
  } catch {
    return { items: [], total: 0 };
  }
}

/**
 * Create or update a custom mail template.
 *
 * @author coisini
 * @created 2026-07-22
 *
 * @param input - Template save payload
 * @returns Updated template list
 */
export async function mailTemplateSave(
  input: MailIpcTemplateSaveRequest,
): Promise<{ items: MailTemplate[]; total: number }> {
  const response = await invokeIpc<MailIpcTemplateListResponse>("mail_template_save", {
    request: input,
  });
  try {
    const parsed = JSON.parse(response.templates_json ?? "[]") as MailTemplate[];
    return { items: Array.isArray(parsed) ? parsed : [], total: response.total ?? 0 };
  } catch {
    return { items: [], total: 0 };
  }
}

/**
 * Apply a template for one customer.
 *
 * @author coisini
 * @created 2026-07-21
 */
export async function mailTemplateApply(
  input: MailIpcTemplateApplyRequest,
): Promise<MailIpcTemplateApplyResponse> {
  return invokeIpc<MailIpcTemplateApplyResponse>("mail_template_apply", { request: input });
}

/**
 * Load saved mail accounts.
 *
 * @author coisini
 * @created 2026-07-21
 */
export async function mailAccountList(): Promise<{ items: MailAccount[]; total: number }> {
  const response = await invokeIpc<MailIpcAccountListResponse>("mail_account_list");
  try {
    const parsed = JSON.parse(response.accounts_json ?? "[]") as MailAccount[];
    return { items: Array.isArray(parsed) ? parsed : [], total: response.total ?? 0 };
  } catch {
    return { items: [], total: 0 };
  }
}

/**
 * Save one mail account.
 *
 * @author coisini
 * @created 2026-07-21
 */
export async function mailAccountSave(
  input: MailIpcAccountSaveRequest,
): Promise<{ items: MailAccount[]; total: number }> {
  const response = await invokeIpc<MailIpcAccountListResponse>("mail_account_save", {
    request: input,
  });
  try {
    const parsed = JSON.parse(response.accounts_json ?? "[]") as MailAccount[];
    return { items: Array.isArray(parsed) ? parsed : [], total: response.total ?? 0 };
  } catch {
    return { items: [], total: 0 };
  }
}

/**
 * Send one outbound message via SMTP.
 *
 * @author coisini
 * @created 2026-07-21
 */
export async function mailSend(input: MailIpcSendRequest): Promise<MailIpcSendResponse> {
  return invokeIpc<MailIpcSendResponse>("mail_send", { request: input });
}

/**
 * Record one inbound message manually.
 *
 * @author coisini
 * @created 2026-07-21
 */
export async function mailRecordInbound(
  input: MailIpcRecordInboundRequest,
): Promise<MailIpcRecordInboundResponse> {
  return invokeIpc<MailIpcRecordInboundResponse>("mail_record_inbound", { request: input });
}

/**
 * List local inbox/sent messages for the mail workbench.
 *
 * @author coisini
 * @created 2026-07-22
 *
 * @param input - Direction and optional filters
 * @returns Message items and total count
 */
export async function mailMessageList(
  input: MailIpcMessageListRequest,
): Promise<{ items: MailMessage[]; total: number }> {
  const response = await invokeIpc<MailIpcMessageListResponse>("mail_message_list", {
    request: input,
  });
  try {
    const parsed = JSON.parse(response.messages_json ?? "[]") as MailMessage[];
    return { items: Array.isArray(parsed) ? parsed : [], total: response.total ?? 0 };
  } catch {
    return { items: [], total: 0 };
  }
}

/**
 * Enqueue IMAP sync background jobs.
 *
 * @author coisini
 * @created 2026-07-22
 */
export async function mailSyncNow(
  input?: MailIpcSyncNowRequest,
): Promise<{ jobIds: string[]; enqueued: number }> {
  const response = await invokeIpc<MailIpcSyncNowResponse>("mail_sync_now", {
    request: input ?? {},
  });
  try {
    const parsed = JSON.parse(response.job_ids_json ?? "[]") as string[];
    return {
      jobIds: Array.isArray(parsed) ? parsed : [],
      enqueued: response.enqueued ?? 0,
    };
  } catch {
    return { jobIds: [], enqueued: 0 };
  }
}

/**
 * Read IMAP sync status for one or all accounts.
 *
 * @author coisini
 * @created 2026-07-22
 */
export async function mailSyncStatus(
  input?: MailIpcSyncStatusRequest,
): Promise<{ items: MailImapSyncState[]; total: number }> {
  const response = await invokeIpc<MailIpcSyncStatusResponse>("mail_sync_status", {
    request: input ?? {},
  });
  try {
    const parsed = JSON.parse(response.items_json ?? "[]") as MailImapSyncState[];
    return { items: Array.isArray(parsed) ? parsed : [], total: response.total ?? 0 };
  } catch {
    return { items: [], total: 0 };
  }
}

/**
 * List inbound messages without a linked customer.
 *
 * @author coisini
 * @created 2026-07-22
 */
export async function mailInboxUnmatchedList(
  input?: MailIpcInboxUnmatchedListRequest,
): Promise<{ items: MailMessage[]; total: number }> {
  const response = await invokeIpc<MailIpcInboxUnmatchedListResponse>("mail_inbox_unmatched_list", {
    request: input ?? {},
  });
  try {
    const parsed = JSON.parse(response.messages_json ?? "[]") as MailMessage[];
    return { items: Array.isArray(parsed) ? parsed : [], total: response.total ?? 0 };
  } catch {
    return { items: [], total: 0 };
  }
}

/**
 * Mark one local message as read.
 *
 * @author coisini
 * @created 2026-08-01
 */
export async function mailMessageMarkRead(
  input: MailIpcMessageMarkReadRequest,
): Promise<{ messageId: string; isRead: boolean }> {
  const response = await invokeIpc<MailIpcMessageMarkReadResponse>("mail_message_mark_read", {
    request: input,
  });
  return { messageId: response.message_id, isRead: response.is_read };
}

/**
 * Link one unmatched inbound message to a customer.
 *
 * @author coisini
 * @created 2026-07-22
 */
export async function mailLinkInboundCustomer(
  input: MailIpcLinkInboundCustomerRequest,
): Promise<{ messageId: string }> {
  const response = await invokeIpc<MailIpcLinkInboundCustomerResponse>("mail_link_inbound_customer", {
    request: input,
  });
  return { messageId: response.message_id };
}

/**
 * Generate styled HTML email from plain text via configured LLM.
 *
 * @author coisini
 * @created 2026-08-01
 */
export async function mailGenerateHtml(
  input: MailIpcGenerateHtmlRequest,
): Promise<{ html: string; notes?: string | null }> {
  const response = await invokeIpc<MailIpcGenerateHtmlResponse>("mail_generate_html", {
    request: input,
  });
  return { html: response.html, notes: response.notes ?? null };
}
