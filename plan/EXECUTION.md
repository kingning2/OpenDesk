# OpenDesk Execution Plan

Architecture Skeleton 阶段执行索引 — **Current** 与 **Next** 为单一真相源。

## Current

_(无 — M2/M3 基建任务已全部完成)_

## Next

_(待定 — 可新增 M4 业务 Feature 任务)_

---

## Queue

| ID | Task | Priority | Status | Milestone | Depends on |
|----|------|----------|--------|-----------|------------|
| T001 | [architecture-compliance](./completed/T001-architecture-compliance.md) | P0 | completed | M1 | — |
| T002 | [contract-codegen](./completed/T002-contract-codegen.md) | P0 | completed | M1 | T001 |
| T003 | [agent-ping-e2e](./completed/T003-agent-ping-e2e.md) | P0 | completed | M1 | T002 |
| T004 | [sidecar-lifecycle](./completed/T004-sidecar-lifecycle.md) | P0 | completed | M1 | T003 |
| T005 | [tracing-log-pipe](./completed/T005-tracing-log-pipe.md) | P0 | completed | M1 | T004 |
| T006 | [sidecar-auto-restart-event](./completed/T006-sidecar-auto-restart-event.md) | P0 | completed | M2 | T005 |
| T007 | [kernel-event-bus](./completed/T007-kernel-event-bus.md) | P1 | completed | M2 | T004 |
| T008 | [kernel-task-scheduler](./completed/T008-kernel-task-scheduler.md) | P1 | completed | M2 | T007 |
| T009 | [storage-record-store](./completed/T009-storage-record-store.md) | P1 | completed | M2 | T001 |
| T010 | [sidecar-management-api](./completed/T010-sidecar-management-api.md) | P1 | completed | M2 | T004 |
| T011 | [frontend-routes-scaffold](./completed/T011-frontend-routes-scaffold.md) | P1 | completed | M3 | T001 |
| T012 | [agent-feature-vertical-slice](./completed/T012-agent-feature-vertical-slice.md) | P1 | completed | M3 | T003, T011 |

---

## Milestones

### M1 — Skeleton Baseline ✅

目录、契约、首条 E2E、sidecar 生命周期、结构化日志接管。

### M2 — Core Runtime ✅

Sidecar 自愈、内核 event/task、storage port、sidecar 管理面客户端。

### M3 — First Feature Template ✅

前端路由骨架 + agent Feature 垂直切片模板。

### M4 — Business Features _(未规划)_

chat / mail / knowledge 等业务 Feature 垂直切片。

---

## 状态目录

| 目录 | 说明 |
|------|------|
| [`pending/`](./pending/) | 待处理 |
| [`in-progress/`](./in-progress/) | 进行中（同时最多 1 个） |
| [`completed/`](./completed/) | 已完成 |

工作流见 [`.cursor/rules/plan-execution.mdc`](../.cursor/rules/plan-execution.mdc)。
