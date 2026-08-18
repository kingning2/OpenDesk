/**
 * 商品管理 IPC 封装 — 列表 / 详情 / 更新。
 *
 * 后端：壳层 `commands/item.rs`（InMemoryItemStore + app::item::ItemService）。
 *
 * @author agent
 * @created 2026-08-13
 */

import { call } from "./invoke";

/** 商品（与 Rust `app::item::Item` 对齐）。 */
export interface Item {
  id: number;
  owner_id: number;
  account_id: string;
  item_id: string;
  title: string;
  price: number;
  desc: string;
  is_polished: boolean;
  is_multi_spec: boolean;
  multi_quantity_delivery: boolean;
  ai_prompt: string;
  has_card: boolean;
  has_default_reply: boolean;
  created_at?: string | null;
}

/** 商品列表查询入参。 */
export interface ItemListRequest {
  owner_id: number;
  page: number;
  page_size: number;
  keyword?: string;
  account_id?: string;
  is_polished?: boolean;
  is_multi_spec?: boolean;
}

/** 查询商品列表（返回 [列表, 总数]）。 */
export function itemList(request: ItemListRequest): Promise<[Item[], number]> {
  return call<[Item[], number]>("item_list", {
    request: {
      owner_id: request.owner_id,
      page: request.page,
      page_size: request.page_size,
      keyword: request.keyword ?? "",
      account_id: request.account_id ?? "",
      is_polished: request.is_polished,
      is_multi_spec: request.is_multi_spec,
    },
  });
}

/** 按商品 ID 查询。 */
export function itemGet(ownerId: number, itemId: string): Promise<Item | null> {
  return call<Item | null>("item_get", { ownerId, itemId });
}

/** 更新商品（AI 提示词）。 */
export function itemUpdate(
  ownerId: number,
  itemId: string,
  aiPrompt?: string,
): Promise<void> {
  return call<void>("item_update", {
    request: { owner_id: ownerId, item_id: itemId, ai_prompt: aiPrompt },
  });
}
