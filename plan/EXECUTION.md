# OpenDesk Execution Plan

Architecture Skeleton 阶段执行索引 — **Current** 与 **Next** 为单一真相源。

## Current

_(无 — T018 P2–P5 已完成，P6 签名/公证待后续)_

## Next

_(P6 签名与公证，或 M4 业务 Feature 垂直切片)_

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
| T013 | [ui-layout-primitives](./completed/T013-ui-layout-primitives.md) | P0 | completed | M5 | T011 |
| T014 | [custom-titlebar](./completed/T014-custom-titlebar.md) | P0 | completed | M5 | T013 |
| T015 | [app-header-bar](./completed/T015-app-header-bar.md) | P1 | completed | M5 | T013, T014 |
| T016 | [shell-refactor-nav-registry](./completed/T016-shell-refactor-nav-registry.md) | P1 | completed | M5 | T013, T014, T015 |
| T017 | [feature-page-scaffold](./completed/T017-feature-page-scaffold.md) | P1 | completed | M5 | T015, T016 |
| T018 | [sidecar-production-bundle](./completed/T018-sidecar-production-bundle.md) | P0 | completed | M6 | T004 |

---

## Milestones

### M1 — Skeleton Baseline ✅

目录、契约、首条 E2E、sidecar 生命周期、结构化日志接管。

### M2 — Core Runtime ✅

Sidecar 自愈、内核 event/task、storage port、sidecar 管理面客户端。

### M3 — First Feature Template ✅

前端路由骨架 + agent Feature 垂直切片模板。

### M4 — Business Features _(未规划)_

chat / mail / knowledge 等业务 Feature 垂直切片（Contract → Rust → Python → React）。

### M5 — Frontend UI Shell ✅

布局原语、Tauri 自定义标题栏、应用 Header、Shell 重构、Feature 占位页骨架。

```
┌─────────────────────────────────────────────┐
│ TitleBar（窗口级：拖拽 + 最小化/最大化/关闭）   │
├──────────┬──────────────────────────────────┤
│ Sidebar  │ AppHeader（页面级：标题 + 主题切换） │
│          ├──────────────────────────────────┤
│          │ PageScaffold / Feature Content   │
└──────────┴──────────────────────────────────┘
```

| 任务 | 交付物 |
|------|--------|
| T013 | `@desk/ui` AppLayout / Sidebar / MainPanel / PageContainer |
| T014 | 无边框窗口 + TitleBar + platform window API |
| T015 | AppHeader + ThemeToggle + 路由标题映射 |
| T016 | Shell 组装 + Feature 导航注册表 |
| T017 | PageScaffold + chat/mail/knowledge 占位路由 |

### M6 — Production Bundle _(进行中)_

Sidecar 冻结 + Tauri externalBin + Win/macOS 签名公证 + CI 集成（T018）。

---

## 状态目录

| 目录 | 说明 |
|------|------|
| [`pending/`](./pending/) | 待处理 |
| [`in-progress/`](./in-progress/) | 进行中（同时最多 1 个） |
| [`completed/`](./completed/) | 已完成 |

工作流见 [`.cursor/rules/plan-execution.mdc`](../.cursor/rules/plan-execution.mdc)。
