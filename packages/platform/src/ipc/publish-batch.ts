/**
 * 批量发布 IPC 封装 — 提交任务 + 查询进度。
 *
 * 后端：壳层 `commands/publish_batch.rs`（InMemoryBatchStore + app::publish::BatchService）。
 *
 * @author agent
 * @created 2026-08-13
 */

import { call } from "./invoke";

/** 单账号发布统计（与 Rust `BatchAccountStatus` 对齐）。 */
export interface BatchAccountStatus {
  account_id: string;
  total: number;
  success: number;
  failed: number;
  publishing: number;
  pending: number;
  sync_status: string;
  sync_message: string;
  sync_total_count: number;
  sync_saved_count: number;
}

/** 批量发布任务（与 Rust `BatchTask` 对齐）。 */
export interface BatchTask {
  batch_id: string;
  owner_id: number;
  account_ids: string[];
  material_ids: number[];
  total: number;
  success: number;
  failed: number;
  publishing: number;
  pending: number;
  finished: boolean;
  account_statuses: BatchAccountStatus[];
}

const OWNER_ID = 1; // 桌面单用户；多用户时由登录态注入

/** 提交批量发布任务（后台执行，返回任务快照）。 */
export function publishBatchSubmit(
  accountIds: string[],
  materialIds: number[],
): Promise<BatchTask> {
  return call<BatchTask>("publish_batch_submit", {
    request: { owner_id: OWNER_ID, account_ids: accountIds, material_ids: materialIds },
  });
}

/** 查询批量发布进度。 */
export function publishBatchStatus(batchId: string): Promise<BatchTask | null> {
  return call<BatchTask | null>("publish_batch_status", {
    request: { owner_id: OWNER_ID, batch_id: batchId },
  });
}
