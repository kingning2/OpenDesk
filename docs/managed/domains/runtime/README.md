# Runtime / Worker Domain

## 职责

桌面端 **进程编排** 与 **重任务 Worker** 生命周期（不含 Python Sidecar，Sidecar 见 Python Runtime 领域）。

- `opendesk-worker` 独立二进制：消费 `background_job` 队列
- Tauri 主进程：入队、spawn/monitor Worker、转发 Event 到 UI
- Job 进度/完成/失败协议（Contract + Tauri Event）
- 与 Storage 协同：`opendesk.db` WAL 双进程访问

## 非职责

- OCR 业务规则细节（OCR 领域；本领域只管队列与进程）
- Python Sidecar 生命周期（已有 runtime/sidecar 实现）
- 爬虫 in-process 执行（Crawler 领域；未来迁 Worker 为独立 Change）
- UI 组件（React）

## 稳定边界

```text
React
  → runtime/job IPC（enqueue, status, cancel）
  → Tauri 主进程 RuntimeModule
  → INSERT background_job
  → spawn opendesk-worker（若未运行）

opendesk-worker
  → poll background_job (queued)
  → dispatch by job_type → OCR / … handlers
  → UPDATE status + progress
  → 写业务结果表（如 ocr_*）

禁止：Worker → WebView；主进程 → OCR 引擎
```

## 入口

| 类型 | 路径（规划） |
|------|--------------|
| Worker 二进制 | `crates/worker/` |
| 主进程编排 | `crates/runtime/` 或 `crates/app/src/worker/` |
| Contract | `contracts/schema/v1/runtime/` |
| 进程模型 | [`docs/architecture/process-model.md`](../../../architecture/process-model.md) |
| ADR | [ADR-0002-heavy-work-worker-process](../../decisions/runtime/adr-0002-heavy-work-worker-process.md) |
| Change | [CHG-023](../../changes/2026/07/chg-20260720-023-opendesk-db-worker-skeleton.md) |

## job_type（MVP 规划）

| job_type | 执行进程 | Handler |
|----------|----------|---------|
| `ocr` | Worker | OCR 引擎（CHG-024） |
| `imap_sync` | Worker | IMAP 收信（CHG-029） |
| `mail_send` | 主进程 async（短连接） | 非 Worker；预留字段一致性 |
| `crawler_batch` | 预留 | 未来爬虫迁 Worker |

## 当前状态

**未实现。** Sidecar 生命周期已有；`opendesk-worker` 与 `background_job` 未实现。

设计已完成：[CHG-022](../../changes/2026/07/chg-20260720-022-database-worker-ocr-design.md)。

## 当前约束

- 重 CPU/IO 任务 **不得** 在 Tauri 主进程执行（ADR-0002）
- Worker 与主进程共享 `opendesk.db`，须 WAL
- UI 更新仅经 DB 状态 + Event，Worker 不回调 WebView
