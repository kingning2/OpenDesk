/**
 * 单品发布 IPC 封装 — 账号能力检测 + 发布执行。
 *
 * 后端：壳层 `commands/publish.rs`（InMemoryPublishGateway + app::publish::PublishService）。
 *
 * @author agent
 * @created 2026-08-13
 */

import { call } from "./invoke";

/** 账号发布能力（与 Rust `AccountCapability` 对齐）。 */
export interface PublishAccountCapability {
  success: boolean;
  is_fish_shop: boolean;
  cookies_str: string | null;
  message: string;
}

/** 商品同步信息（与 Rust `SyncInfo` 对齐）。 */
export interface PublishSyncInfo {
  sync_status: string;
  sync_message: string;
  sync_total_count: number;
  sync_saved_count: number;
}

/** 发布结果（与 Rust `PublishServiceResult` 对齐）。 */
export interface PublishResult {
  success: boolean;
  message: string;
  item_url: string | null;
  item_id: string | null;
  log_id: number;
  sync: PublishSyncInfo;
}

const OWNER_ID = 1; // 桌面单用户；多用户时由登录态注入

/** 账号发布能力检测。 */
export function publishCapability(accountId: string): Promise<PublishAccountCapability> {
  return call<PublishAccountCapability>("publish_capability", {
    request: { user_id: OWNER_ID, account_id: accountId },
  });
}

/** 执行单品发布（item 为商品数据对象）。 */
export function publishSingle(
  accountId: string,
  item: Record<string, unknown>,
): Promise<PublishResult> {
  return call<PublishResult>("publish_single", {
    request: { user_id: OWNER_ID, account_id: accountId, item },
  });
}
