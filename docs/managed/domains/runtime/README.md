# Runtime / Worker Domain

## 职责

桌面端 **进程编排** 与 **重任务 Worker** 生命周期。当前实现聚焦 **Python Sidecar 生命周期**：

- `SidecarLifecycle`：启动、监控、退出 Python sidecar 进程（`crates/runtime/src/sidecar/lifecycle.rs`）
- `SidecarClient` + 路由：`agent_ping` / `manage`（`crates/runtime/src/sidecar/routes/`）
- 日志管道：`log_pipe.rs`
- 配置：`SidecarConfig::from_env()`

**重任务 Worker（dingda-worker）已随采集/客户/邮箱板块移除**，如后续需要按新 Change 重新引入。

## 非职责

- OCR 业务规则细节（OCR 领域；规划中）
- Python Sidecar 内部实现（Python Runtime 领域）
- 领域业务规则（业务代码在 `apps/desktop/src-tauri`）

## 稳定边界

```text
Tauri 主进程（src-tauri）
  → runtime::SidecarLifecycle（spawn / monitor / stop）
  → Python sidecar 进程
  → sidecar routes（agent_ping …）
  → Event → React
```

## 入口

| 类型 | 路径 |
|------|------|
| Crate | `crates/runtime/` |
| Lifecycle | `crates/runtime/src/sidecar/lifecycle.rs` |
| Client | `crates/runtime/src/sidecar/client.rs` |
| ADR | [ADR-0002-heavy-work-worker-process](../../decisions/runtime/adr-0002-heavy-work-worker-process.md) |

## 当前状态

Sidecar 生命周期已实现并接入应用（`src-tauri/src/lib.rs`）。重任务 Worker 编排为规划，未实现。

## 当前约束

- 重 CPU/IO 任务 **不得** 在 Tauri 主进程执行（ADR-0002）
- Sidecar 进程由 Rust 唯一编排，UI 不直连 Python
