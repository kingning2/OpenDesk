/**
 * 闲鱼关键词搜索 IPC。
 *
 * Sidecar 经 Camoufox/Chromium 拦截 MTOP idlemtopsearch 搜索结果。
 *
 * @author Xiaoman
 * @created 2026-08-22
 */

import { callRequest } from "./invoke";

/** 单条搜索结果（与 sidecar offer 对齐）。 */
export interface XianyuSearchItem {
  itemId: string;
  title: string;
  url: string;
  image?: string;
  price?: { text?: string; original?: string };
  location?: string;
  seller?: { name?: string };
  wantCount?: string;
  publishedAt?: string;
  tags?: string[];
}

/** 搜索响应。 */
export interface XianyuSearchResponse {
  ok: boolean;
  status: string;
  keyword: string;
  total: number;
  totalBeforeFilter: number;
  offers: XianyuSearchItem[];
  finalUrl?: string;
  detail: string;
}

/** 闲鱼关键词搜索。默认有头浏览器，便于过滑块。 */
export function xianyuSearch(params: {
  ownerId: number;
  accountId: string;
  keyword: string;
  maxResults?: number;
  headed?: boolean;
}): Promise<XianyuSearchResponse> {
  return callRequest<XianyuSearchResponse>("xianyu_search", {
    request: {
      owner_id: params.ownerId,
      account_id: params.accountId,
      keyword: params.keyword,
      max_results: params.maxResults,
      headed: params.headed ?? true,
    },
  }).then((response) => response.data);
}
