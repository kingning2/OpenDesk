# Workflow Runtime Domain

## 职责

通用工作流**执行运行时**（非 outreach 话术配置）：

- DAG 定义校验与调度
- 节点 Executor 注册与执行
- Workflow / Node 状态机
- 共享 Context
- SQLite 检查点与 Resume / Pause / Cancel / Retry
- EventBus（供 UI 订阅）

## 非职责

- React Flow 画布编辑 UX（桌面 Feature）
- Outreach 阶段 / 话术库（见 `crates/workflow` / domain `workflow`）
- 具体业务 I/O（HTTP/AI/Crawler 等在各自 Executor 内）

## 稳定边界

```text
React Flow Definition
  → IPC workflow_runtime/*
  → crates/workflow_runtime Facade
  → Scheduler + Registry + CheckpointStore
  → ports → storage（opendesk.db wf_rt_*）
  → EventBus → Tauri Event → UI
```

## 入口

| 类型 | 路径 |
|------|------|
| ADR | [ADR-0007](../../decisions/workflow-runtime/adr-0007-workflow-runtime.md) |
| Rust | `crates/workflow_runtime/` |
| Port | `crates/ports/src/workflow_runtime.rs` |
| Storage | `crates/storage/src/workflow_runtime/` |
| Contract | `contracts/schema/v1/workflow_runtime/` |

## 当前状态

首版 Runtime 骨架落地中（CHG-20260723-002）。
