/**
 * 黑名单管理 IPC 封装 — 个人黑名单 CRUD + 平台黑名单查询。
 *
 * 后端：壳层 `commands/blacklist.rs`（InMemoryBlacklistStore + app::blacklist::BlacklistService）。
 *
 * @author agent
 * @created 2026-08-13
 */

import { call } from "./invoke";

/** 个人黑名单条目（与 Rust `PersonalBlacklistItem` 对齐）。 */
export interface PersonalBlacklistItem {
  id: number;
  owner_id: number;
  account_id: string | null;
  buyer_id: string;
  buyer_nick: string | null;
  item_id: string | null;
  reason: string | null;
  is_enabled: boolean;
  created_at: string | null;
}

/** 命中级别：商品级 > 账户级 > 用户级（与 Rust `level()` 对齐）。 */
export function blacklistLevel(item: PersonalBlacklistItem): string {
  if (item.account_id && item.item_id) return "商品级";
  if (item.account_id) return "账户级";
  return "用户级";
}

/** 平台黑名单条目。 */
export interface PlatformBlacklistItem {
  id: number;
  owner_id: number;
  buyer_id: string;
  buyer_nick: string | null;
  created_at: string | null;
}

/** 查询个人黑名单。 */
export function blacklistPersonalList(request: {
  owner_id: number;
  page: number;
  page_size: number;
  buyer_id?: string;
  buyer_nick?: string;
}): Promise<[PersonalBlacklistItem[], number]> {
  return call<[PersonalBlacklistItem[], number]>("blacklist_personal_list", {
    request: {
      owner_id: request.owner_id,
      page: request.page,
      page_size: request.page_size,
      buyer_id: request.buyer_id ?? "",
      buyer_nick: request.buyer_nick ?? "",
    },
  });
}

/** 查询平台黑名单。 */
export function blacklistPlatformList(ownerId: number): Promise<[PlatformBlacklistItem[], number]> {
  return call<[PlatformBlacklistItem[], number]>("blacklist_platform_list", { ownerId });
}

/** 新建个人黑名单（buyer_ids 支持逗号/换行分隔批量）。 */
export function blacklistPersonalCreate(request: {
  owner_id: number;
  buyer_ids: string;
  account_id?: string;
  item_id?: string;
  reason?: string;
}): Promise<PersonalBlacklistItem[]> {
  return call<PersonalBlacklistItem[]>("blacklist_personal_create", {
    request: {
      owner_id: request.owner_id,
      buyer_ids: request.buyer_ids,
      account_id: request.account_id,
      item_id: request.item_id,
      reason: request.reason,
    },
  });
}

/** 切换启用状态。 */
export function blacklistSetEnabled(ownerId: number, id: number, enabled: boolean): Promise<void> {
  return call<void>("blacklist_set_enabled", {
    request: { owner_id: ownerId, id, enabled },
  });
}

/** 删除。 */
export function blacklistDelete(ownerId: number, id: number): Promise<void> {
  return call<void>("blacklist_delete", { request: { owner_id: ownerId, id } });
}
