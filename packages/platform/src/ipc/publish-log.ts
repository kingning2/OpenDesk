/**
 * 发布日志 IPC 封装 — 日志分页查询 + 清空。
 *
 * 后端：壳层 `commands/publish_log.rs`（InMemoryPublishLogStore + app::publish::PublishLogService）。
 *
 * @author agent
 * @created 2026-08-13
 */

import { call } from "./invoke";

/** 发布日志条目（与 Rust `PublishLog` 对齐核心字段）。 */
export interface PublishLog {
  id: number;
  owner_id: number;
  account_id: string;
  title: string;
  price: string | null;
  status: "pending" | "publishing" | "success" | "failed";
  item_url: string | null;
  item_id: string | null;
  error_message: string | null;
  resolved_address_text: string | null;
  address_source: string | null;
  created_at: string | null;
}

const OWNER_ID = 1; // 桌面单用户；多用户时由登录态注入

/** 分页查询发布日志。 */
export function publishLogList(query: {
  page: number;
  page_size: number;
  account_id?: string;
  status?: string;
}): Promise<[PublishLog[], number]> {
  return call<[PublishLog[], number]>("publish_log_list", {
    request: {
      owner_id: OWNER_ID,
      page: query.page,
      page_size: query.page_size,
      account_id: query.account_id ?? "",
      status: query.status ?? "",
    },
  });
}

/** 清空 N 天前的日志（0 = 全部）。 */
export function publishLogClear(days = 10): Promise<void> {
  return call<void>("publish_log_clear", {
    request: { owner_id: OWNER_ID, days },
  });
}
