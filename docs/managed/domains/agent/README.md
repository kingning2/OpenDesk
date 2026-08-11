# Agent Domain

## 职责

Agent 领域负责 AI 规划、模型交互与任务执行，是当前桌面应用的**核心业务**。Rust 侧通过 Sidecar Gateway 与 Python sidecar 通信，Python 承担 AI 推理。

当前实现（骨架）：

- **PingAgent**：通过 `AgentSidecarGateway.ping()` 探活 Python sidecar（见 [`src-tauri/src/agent.rs`](../../../../../apps/desktop/src-tauri/src/agent.rs)）
- **License 门禁**：授权校验在应用入口处（见 `state.rs` 的 `build_license_gate`）

## 非职责

- 直接修改业务持久化（禁止写库工具；写操作仅 UI 人工操作 → Rust IPC）
- AI 自动发送消息
- Python 直连 SQLite
- 绕过 Rust 直接执行高权限操作

## 稳定边界

```text
React → Rust agent IPC（src-tauri/src/commands/agent.rs）
         ↓ PingAgent
    AgentSidecarGateway（ports::sidecar）
         ↓ sidecar_request
    Python gateway（agent_ping handler）
```

## 有效 ADR

- [ADR-0001-ai-readonly-query-port](../../decisions/customer/adr-0001-ai-readonly-query-port.md)
- [ADR-0005-ai-correction-memory](../../decisions/agent/adr-0005-ai-correction-memory.md)

## 入口

| 类型 | 路径 |
|------|------|
| Rust（业务） | `apps/desktop/src-tauri/src/agent.rs`、`commands/agent.rs` |
| Gateway trait | `crates/ports/src/sidecar.rs` |
| Adapter | `crates/adapter/`（`RuntimeAgentSidecar`） |
| Runtime | `crates/runtime/`（`SidecarLifecycle`） |
| Python | `python/packages/gateway/`（`agent_ping` handler） |
| Contract | `contracts/`（`agent_ipc_ping_*` / `agent_sidecar_ping_*`） |

## 当前状态

PingAgent 骨架已实现，应用可通过 IPC 探活 Python sidecar。后续 AI 能力（LLM 推理、只读 Query 工具、纠错记忆等）按新 Change 逐项实施。
