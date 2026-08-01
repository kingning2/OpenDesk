---
id: ADR-0007-workflow-runtime
title: 独立 Workflow Runtime crate（DAG + 检查点 + Executor Registry）
status: accepted
domain: workflow-runtime
created: 2026-07-23
supersedes: none
---

# 独立 Workflow Runtime crate（DAG + 检查点 + Executor Registry）

## Context

OpenDesk 需要可扩展的工作流执行：React Flow 编辑图，Rust 调度执行，并支持进程退出后恢复。现有 `crates/workflow` 仅承载 outreach 话术 / 阶段，不宜塞入运行时。采集等 Feature 需要可注册的节点执行器，且 Scheduler 不得包含业务 I/O。

## Decision

1. 新建 **`crates/workflow_runtime`**，与 outreach `crates/workflow` 命名隔离。
2. Scheduler 只做就绪判定、并发、Retry、状态迁移；副作用仅在 `NodeExecutor` 内。
3. 新增节点 = 实现 `NodeExecutor` + `ExecutorRegistry::register`，禁止改 Scheduler。
4. 状态一律 `enum` + `match`；错误一律 `WorkflowError`；禁止魔法 status 字符串/数字驱动分支。
5. 持久化表前缀 `wf_rt_*`，落 `opendesk.db`，经 `ports::workflow_runtime::CheckpointStore`；每节点完成即事务落盘。
6. UI 只订 EventBus / IPC；Executor 禁止直接更新 UI。
7. 首期 DAG **无环**；采集 `auto_loop` 用 Instance 级 `RunPolicy`，不在图内回边。

## Alternatives

| 方案 | 未选原因 |
|------|----------|
| 并入 `crates/workflow` | 与 outreach 话术职责混淆，易成 God Object |
| 仅做采集专用 FSM | 无法复用到 HTTP/AI/Mail 等节点 |
| React Flow 本地执行 | 违反 React→Rust 架构，无法可靠恢复 |

## Consequences

- 正面：可测试、可扩展、可恢复；Feature 解耦。
- 成本：需维护独立 crate、migration、Contract IPC。
- 兼容：outreach `script_snippet` 表与 API 不变。
