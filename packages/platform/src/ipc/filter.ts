/**
 * 消息过滤规则 IPC 封装 — 过滤规则 CRUD + 启用切换。
 *
 * 后端：壳层 `commands/filter.rs`（InMemoryFilterStore + app::auto_reply::FilterService）。
 *
 * @author agent
 * @created 2026-08-13
 */

import { call } from "./invoke";

/** 过滤类型（与 Rust `FilterType` 对齐）。 */
export type FilterType = "skip_reply" | "skip_notify";

/** 过滤规则（与 Rust `FilterRule` 对齐）。 */
export interface FilterRule {
  id: number;
  account_id: string;
  owner_id: number;
  filter_type: FilterType;
  keyword: string;
  enabled: boolean;
}

/** 过滤类型中文文案。 */
export const FILTER_TYPE_LABELS: Record<FilterType, string> = {
  skip_reply: "跳过自动回复",
  skip_notify: "跳过消息通知",
};

/** 过滤类型选项（供表单使用）。 */
export const FILTER_TYPE_OPTIONS: { value: FilterType; label: string }[] = [
  { value: "skip_reply", label: FILTER_TYPE_LABELS.skip_reply },
  { value: "skip_notify", label: FILTER_TYPE_LABELS.skip_notify },
];

/** 查询账号下的过滤规则。 */
export function filterList(ownerId: number, accountId: string): Promise<FilterRule[]> {
  return call<FilterRule[]>("filter_list", {
    request: { owner_id: ownerId, account_id: accountId },
  });
}

/** 新建过滤规则。 */
export function filterCreate(
  ownerId: number,
  accountId: string,
  rule: Pick<FilterRule, "filter_type" | "keyword">,
): Promise<FilterRule> {
  return call<FilterRule>("filter_create", {
    request: {
      owner_id: ownerId,
      account_id: accountId,
      rule: { ...rule, id: 0, account_id: accountId, owner_id: ownerId, enabled: true },
    },
  });
}

/** 更新过滤规则。 */
export function filterUpdate(
  ownerId: number,
  rule: Pick<FilterRule, "id" | "filter_type" | "keyword" | "account_id">,
): Promise<void> {
  return call<void>("filter_update", {
    request: {
      owner_id: ownerId,
      rule: { ...rule, owner_id: ownerId, enabled: true },
    },
  });
}

/** 切换启用状态。 */
export function filterSetEnabled(ownerId: number, ruleId: number, enabled: boolean): Promise<void> {
  return call<void>("filter_set_enabled", {
    request: { owner_id: ownerId, rule_id: ruleId, enabled },
  });
}

/** 删除过滤规则。 */
export function filterDelete(ownerId: number, ruleId: number): Promise<void> {
  return call<void>("filter_delete", {
    request: { owner_id: ownerId, rule_id: ruleId },
  });
}
