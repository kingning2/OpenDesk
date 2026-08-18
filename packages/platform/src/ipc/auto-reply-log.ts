/**
 * 自动回复日志 IPC 封装 — 回复明细分页查询。
 *
 * 后端：壳层 `commands/auto_reply_log.rs`（InMemoryAutoReplyLogStore + app::auto_reply::AutoReplyLogService）。
 *
 * @author agent
 * @created 2026-08-13
 */

import { call } from "./invoke";

/** 自动回复日志条目（与 Rust `AutoReplyLogItem` 对齐）。 */
export interface AutoReplyLogItem {
  id: number;
  owner_id: number | null;
  owner_username: string | null;
  account_pk: number | null;
  account_id: string;
  account_name: string | null;
  chat_id: string;
  item_id: string | null;
  item_title: string | null;
  order_no: string | null;
  source_message_id: string | null;
  sender_user_id: string;
  sender_user_name: string | null;
  source_message: string | null;
  source_message_time: string | null;
  process_status: string;
  decision_reason: string;
  reply_strategy: string;
  reply_mode: string;
  matched_keyword: string | null;
  matched_rule_type: string | null;
  default_reply_scope: string | null;
  default_reply_once: boolean;
  ai_model_name: string | null;
  ai_provider_name: string | null;
  reply_text: string | null;
  reply_image_url: string | null;
  error_message: string | null;
  send_status: string;
  send_fail_reason: string | null;
  created_at: string | null;
  updated_at: string | null;
}

/** 分页结果（与 Rust `AutoReplyLogPage` 对齐）。 */
export interface AutoReplyLogPage {
  data: AutoReplyLogItem[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

/** 日志查询条件。 */
export interface AutoReplyLogQuery {
  page: number;
  page_size: number;
  account_id?: string;
  start_date?: string;
  end_date?: string;
  matched_rule_type?: string;
  send_status?: string;
  message_type?: string;
}

/** 分页查询自动回复日志。 */
export function autoReplyLogList(query: AutoReplyLogQuery): Promise<AutoReplyLogPage> {
  return call<AutoReplyLogPage>("auto_reply_log_list", {
    request: {
      owner_id: 1, // 桌面单用户；多用户时由登录态注入
      page: query.page,
      page_size: query.page_size,
      account_id: query.account_id ?? "",
      start_date: query.start_date ?? "",
      end_date: query.end_date ?? "",
      matched_rule_type: query.matched_rule_type ?? "",
      send_status: query.send_status ?? "",
      message_type: query.message_type ?? "",
    },
  });
}
