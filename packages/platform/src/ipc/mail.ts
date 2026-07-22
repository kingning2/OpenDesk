import { invoke } from "@tauri-apps/api/core";
import type {
  MailDtoMailAccount,
  MailDtoMailMessage,
  MailDtoMailTemplate,
  MailIpcAccountListResponse,
  MailIpcAccountSaveRequest,
  MailIpcMessageListRequest,
  MailIpcMessageListResponse,
  MailIpcRecordInboundRequest,
  MailIpcRecordInboundResponse,
  MailIpcSendRequest,
  MailIpcSendResponse,
  MailIpcTemplateApplyRequest,
  MailIpcTemplateApplyResponse,
  MailIpcTemplateListResponse,
  MailIpcTemplateSaveRequest,
} from "@desk/contracts";

export type MailTemplate = MailDtoMailTemplate;
export type MailAccount = MailDtoMailAccount;
export type MailMessage = MailDtoMailMessage;

/**
 * Load all mail templates.
 *
 * @author Xiaoman
 * @created 2026-07-21
 */
export async function mailTemplateList(): Promise<{ items: MailTemplate[]; total: number }> {
  const response = await invoke<MailIpcTemplateListResponse>("mail_template_list");
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
 * @author Xiaoman
 * @created 2026-07-22
 *
 * @param input - Template save payload
 * @returns Updated template list
 */
export async function mailTemplateSave(
  input: MailIpcTemplateSaveRequest,
): Promise<{ items: MailTemplate[]; total: number }> {
  const response = await invoke<MailIpcTemplateListResponse>("mail_template_save", {
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
 * @author Xiaoman
 * @created 2026-07-21
 */
export async function mailTemplateApply(
  input: MailIpcTemplateApplyRequest,
): Promise<MailIpcTemplateApplyResponse> {
  return invoke<MailIpcTemplateApplyResponse>("mail_template_apply", { request: input });
}

/**
 * Load saved mail accounts.
 *
 * @author Xiaoman
 * @created 2026-07-21
 */
export async function mailAccountList(): Promise<{ items: MailAccount[]; total: number }> {
  const response = await invoke<MailIpcAccountListResponse>("mail_account_list");
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
 * @author Xiaoman
 * @created 2026-07-21
 */
export async function mailAccountSave(
  input: MailIpcAccountSaveRequest,
): Promise<{ items: MailAccount[]; total: number }> {
  const response = await invoke<MailIpcAccountListResponse>("mail_account_save", {
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
 * @author Xiaoman
 * @created 2026-07-21
 */
export async function mailSend(input: MailIpcSendRequest): Promise<MailIpcSendResponse> {
  return invoke<MailIpcSendResponse>("mail_send", { request: input });
}

/**
 * Record one inbound message manually.
 *
 * @author Xiaoman
 * @created 2026-07-21
 */
export async function mailRecordInbound(
  input: MailIpcRecordInboundRequest,
): Promise<MailIpcRecordInboundResponse> {
  return invoke<MailIpcRecordInboundResponse>("mail_record_inbound", { request: input });
}

/**
 * List local inbox/sent messages for the mail workbench.
 *
 * @author Xiaoman
 * @created 2026-07-22
 *
 * @param input - Direction and optional filters
 * @returns Message items and total count
 */
export async function mailMessageList(
  input: MailIpcMessageListRequest,
): Promise<{ items: MailMessage[]; total: number }> {
  const response = await invoke<MailIpcMessageListResponse>("mail_message_list", {
    request: input,
  });
  try {
    const parsed = JSON.parse(response.messages_json ?? "[]") as MailMessage[];
    return { items: Array.isArray(parsed) ? parsed : [], total: response.total ?? 0 };
  } catch {
    return { items: [], total: 0 };
  }
}
