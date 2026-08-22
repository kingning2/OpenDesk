/**
 * 1688 关键词搜索 IPC。
 *
 * Sidecar 经 Camoufox 指纹浏览器拦截 MTOP 搜索结果。
 *
 * @author Xiaoman
 * @created 2026-08-22
 */

import { callRequest } from "./invoke";

/** 单条搜索结果（与 sidecar offer 对齐，嵌套字段保留原样）。 */
export interface Ali1688SearchOffer {
  offerId: string;
  title: string;
  price?: { text?: string; min?: number; max?: number };
  supplier?: { name?: string; shopUrl?: string; years?: number };
  location?: { province?: string; city?: string };
  tags?: string[];
  turnover?: string;
  isP4P?: boolean;
  url: string;
  image?: string;
}

/** 搜索响应。 */
export interface Ali1688SearchResponse {
  ok: boolean;
  status: string;
  keyword: string;
  total: number;
  totalBeforeFilter: number;
  offers: Ali1688SearchOffer[];
  finalUrl?: string;
  detail: string;
}

/** 1688 关键词搜索。默认有头 Camoufox，便于过滑块。 */
export function ali1688Search(params: {
  ownerId: number;
  accountId: string;
  keyword: string;
  maxResults?: number;
  headed?: boolean;
}): Promise<Ali1688SearchResponse> {
  return callRequest<Ali1688SearchResponse>("ali1688_search", {
    request: {
      owner_id: params.ownerId,
      account_id: params.accountId,
      keyword: params.keyword,
      max_results: params.maxResults,
      headed: params.headed ?? true,
    },
  }).then((response) => response.data);
}
