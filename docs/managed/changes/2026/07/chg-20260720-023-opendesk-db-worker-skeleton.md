---
id: CHG-20260720-023-opendesk-db-worker-skeleton
title: opendesk.db 基础与 opendesk-worker 二进制骨架
type: change
status: proposed
priority: P1
owner: developer
domain: storage
parent: EPIC-20260720-001-mvp-sales-workbench
depends_on:
  - CHG-20260720-022-database-worker-ocr-design
blocks:
  - CHG-20260720-024-ocr-worker-pipeline
milestone: M6
created: 2026-07-20
updated: 2026-07-20
contracts: runtime job IPC + events
related:
  - ADR-0002-heavy-work-worker-process
---

# opendesk.db 基础与 opendesk-worker 二进制骨架

## 目标

1. 创建 `opendesk.db` Diesel Migration 基础设施（`crates/storage/src/opendesk_db/`）
2. 首批 Migration：`background_job` 表（OCR 与重任务队列）
3. 新增独立二进制 **`opendesk-worker`**：轮询 `queued` job、更新状态（**M6 先不跑 OCR 引擎**，只跑 noop/health job 验证进程隔离）
4. Tauri 主进程：spawn/monitor Worker、**禁止**在主进程 crate 依赖 OCR 引擎

## 非目标

- OCR 识别实现（CHG-024）
- Customer/Mail 等业务表（仍由 CHG-013/015 等各自 Migration，但共用 `opendesk_db` 模块）
- 爬虫迁 Worker

## 影响与边界

### 修改范围

| 层级 | 路径 | 改动 |
|------|------|------|
| Storage | `crates/storage/src/opendesk_db/mod.rs` | 模块入口 |
| Storage | `crates/storage/src/opendesk_db/schema.rs` | Diesel schema |
| Storage | `crates/storage/migrations/opendesk/` | Migration 目录 |
| Storage | `crates/storage/migrations/opendesk/*_create_background_job/` | 首表 |
| Crate | `crates/worker/`（新建） | Worker 二进制 crate |
| Crate | `crates/worker/src/main.rs` | 队列轮询循环 |
| Crate | `crates/worker/src/job_runner.rs` | claim / complete / fail |
| Crate | `crates/runtime/` 或 `crates/app/` | WorkerLifecycle：spawn、健康检查 |
| Contract | `contracts/schema/v1/runtime/ipc/job_enqueue.request.schema.json` | 入队 |
| Contract | `contracts/schema/v1/runtime/ipc/job_status.request.schema.json` | 查状态 |
| Contract | `contracts/schema/v1/runtime/event/job.progress.schema.json` | 进度事件 |
| Contract | `contracts/schema/v1/runtime/event/job.completed.schema.json` | 完成事件 |
| App | `crates/app/Cargo.toml` | **不得**依赖 OCR 引擎 crate |
| Docs | `database-schema.md` | 若 DDL 微调则同步 |

### 不修改范围

| 路径 | 原因 |
|------|------|
| `crates/ocr/` 引擎实现 | CHG-024 |
| `python/**` | Worker 纯 Rust |
| UI 进程内 OCR 调用 | 禁止 |

### Contract

- 需要 runtime job IPC + Event schema

### 跨层

- React → Tauri → enqueue；Event → React 进度

### 风险

- WAL 双进程：Integration test 主进程 + Worker 同时打开 DB
- Worker 路径：打包时随安装包分发 `opendesk-worker.exe`

## 依赖关系

- 前置：CHG-022（设计定稿）
- 阻塞：CHG-024

## 实施方案

1. 按 [`database-schema.md`](../../../architecture/database-schema.md) 创建 `background_job` Migration。
2. 启动时 `PRAGMA journal_mode=WAL; foreign_keys=ON`。
3. `opendesk-worker`：循环 `claim` → 执行 stub → `complete`；支持 `cancelled`。
4. 主进程 `WorkerLifecycle`：首次 enqueue 时 spawn Worker；崩溃重启策略文档化。
5. 架构检查：新增规则 `crates/app` 不得 depend on `ocr-engine`（占位 crate 名）。

## 验收

- [ ] `opendesk.db` 可创建且含 `background_job`
- [ ] 主进程 enqueue → Worker 处理 → status=completed
- [ ] 主进程 IPC 不阻塞等待 Worker（异步 Event）
- [ ] OCR 引擎 crate 未被 `crates/app` 依赖
- [ ] `check_architecture.py` 通过

## 实际结果

（完成前留空。）

## 后续项

- [CHG-024 OCR Worker 管线](chg-20260720-024-ocr-worker-pipeline.md)
