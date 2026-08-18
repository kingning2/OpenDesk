/**
 * 卡券管理 IPC 封装 — 卡券 CRUD / 启用状态。
 *
 * 后端：壳层 `commands/card.rs`（InMemoryCardStore + app::card::CardService）。
 *
 * @author agent
 * @created 2026-08-13
 */

import { call } from "./invoke";

/** 卡券来源（与 Rust `CardSource` 对齐）。 */
export type CardSource = "own" | "dock_l1" | "dock_l2";

/** 卡券（与 Rust `Card` 对齐）。 */
export interface Card {
  id: number;
  owner_id: number;
  account_id: string;
  name: string;
  /** text / data / api / image。 */
  card_type: string;
  source: CardSource;
  enabled: boolean;
  text_content: string;
  data_content: string;
  image_url: string;
  image_urls: string;
  api_config: string;
  delay_seconds: number;
  description: string;
}

/** 卡券列表查询入参。 */
export interface CardListRequest {
  owner_id: number;
  page: number;
  page_size: number;
  keyword?: string;
  card_type?: string;
}

/** 查询卡券列表（返回 [列表, 总数]）。 */
export function cardList(request: CardListRequest): Promise<[Card[], number]> {
  return call<[Card[], number]>("card_list", {
    request: {
      owner_id: request.owner_id,
      page: request.page,
      page_size: request.page_size,
      keyword: request.keyword ?? "",
      card_type: request.card_type ?? "",
    },
  });
}

/** 新建卡券。 */
export function cardCreate(ownerId: number, card: Card): Promise<Card> {
  return call<Card>("card_create", { ownerId, card });
}

/** 更新卡券。 */
export function cardUpdate(ownerId: number, card: Card): Promise<void> {
  return call<void>("card_update", { ownerId, card });
}

/** 切换卡券启用状态。 */
export function cardSetEnabled(ownerId: number, cardId: number, enabled: boolean): Promise<void> {
  return call<void>("card_set_enabled", {
    request: { owner_id: ownerId, card_id: cardId, enabled },
  });
}

/** 删除卡券。 */
export function cardDelete(ownerId: number, cardId: number): Promise<void> {
  return call<void>("card_delete", { request: { owner_id: ownerId, card_id: cardId } });
}
