/**
 * 自动回复关键词 IPC 封装 — 关键词 CRUD / 整表替换。
 *
 * 后端：壳层 `commands/keyword.rs`（InMemoryKeywordStore + app::auto_reply::KeywordService）。
 *
 * @author agent
 * @created 2026-08-13
 */

import { call } from "./invoke";

/** 关键词规则（与 Rust `KeywordRule` 对齐）。 */
export interface KeywordRule {
  id: number;
  account_id: string;
  /** 多行关键词，每行一个触发词。 */
  keyword: string;
  /** 回复内容（图片关键词时为图片 URL 或发送指令）。 */
  reply: string;
  /** 关联商品 ID（空表示全局规则）。 */
  item_id: string;
  /** text / image。 */
  rule_type: string;
  image_url: string;
  item_title: string;
}

/** 查询账号关键词列表。 */
export function keywordList(accountId: string): Promise<KeywordRule[]> {
  return call<KeywordRule[]>("keyword_list", { request: { account_id: accountId } });
}

/** 整表替换保存关键词。 */
export function keywordReplace(accountId: string, keywords: KeywordRule[]): Promise<void> {
  return call<void>("keyword_replace", { request: { account_id: accountId, keywords } });
}

/** 新增关键词（查重）。 */
export function keywordAdd(accountId: string, rule: KeywordRule): Promise<KeywordRule> {
  return call<KeywordRule>("keyword_add", { request: { account_id: accountId, rule } });
}

/** 删除关键词。 */
export function keywordDelete(ruleId: number): Promise<void> {
  return call<void>("keyword_delete", { request: { rule_id: ruleId } });
}
