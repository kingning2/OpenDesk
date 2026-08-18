/**
 * 意见反馈 IPC 封装 — 本地反馈记录 CRUD。
 *
 * 后端：壳层 `commands/feedback.rs`（InMemoryFeedbackStore + app::feedback::FeedbackService）。
 *
 * @author agent
 * @created 2026-08-13
 */

import { call } from "./invoke";

/** 反馈类型（与 Rust `FeedbackKind` 对齐）。 */
export type FeedbackKind = "feature" | "bug" | "other";

/** 反馈记录（与 Rust `Feedback` 对齐）。 */
export interface Feedback {
  id: number;
  owner_id: number;
  kind: FeedbackKind;
  title: string;
  content: string;
  created_at: string | null;
}

const OWNER_ID = 1; // 桌面单用户；多用户时由登录态注入

/** 分页查询反馈。 */
export function feedbackList(query: {
  page: number;
  page_size: number;
  kind?: FeedbackKind;
  keyword?: string;
}): Promise<[Feedback[], number]> {
  return call<[Feedback[], number]>("feedback_list", {
    request: {
      owner_id: OWNER_ID,
      page: query.page,
      page_size: query.page_size,
      kind: query.kind ?? "",
      keyword: query.keyword ?? "",
    },
  });
}

/** 新建反馈。 */
export function feedbackCreate(
  feedback: Omit<Feedback, "id" | "owner_id">,
): Promise<Feedback> {
  return call<Feedback>("feedback_create", {
    ownerId: OWNER_ID,
    feedback: { ...feedback, id: 0, owner_id: OWNER_ID },
  });
}

/** 删除反馈。 */
export function feedbackDelete(feedbackId: number): Promise<void> {
  return call<void>("feedback_delete", {
    request: { owner_id: OWNER_ID, feedback_id: feedbackId },
  });
}
