# Agent Domain

## 职责

Agent 领域负责 AI 规划、模型交互与任务执行，是当前桌面应用的**核心业务**。默认在 **Rust** 实现（编排、LLM HTTP、只读 Query Port）。Python sidecar 只在 [ADR-0009](../../decisions/python-runtime/adr-0009-python-only-when-rust-insufficient.md) 允许的生态缺口下参与。

当前实现（骨架）：

- **PingAgent**：通过 `AgentSidecarGateway.ping()` 探活 Python sidecar（见 [`src-tauri/src/agent.rs`](../../../../../apps/desktop/src-tauri/src/agent.rs)）。此路径是既有骨架，**不是**「AI 必须在 Python」的依据。
- **License 门禁**：授权校验在应用入口处（见 `state.rs` 的 `build_license_gate`）

## 非职责

- 直接修改业务持久化（禁止写库工具；写操作仅 UI 人工操作 → Rust IPC）
- 把新的 LLM / Agent 能力默认放到 Python
- Python 直连 SQLite
- 绕过 Rust 直接执行高权限操作

## 稳定边界

```text
React → Rust agent IPC（src-tauri/src/commands/agent.rs）
         ↓ 默认在 Rust 完成推理与工具调用
    （仅当 ADR-0009 例外）AgentSidecarGateway
         ↓ sidecar_request
    Python gateway
```

现有 Ping 骨架：

```text
PingAgent → AgentSidecarGateway → Python gateway（agent_ping handler）
```

## 有效 ADR

- [ADR-0009-python-only-when-rust-insufficient](../../decisions/python-runtime/adr-0009-python-only-when-rust-insufficient.md)
- [ADR-0001-ai-readonly-query-port](../../decisions/customer/adr-0001-ai-readonly-query-port.md)
- [ADR-0005-ai-correction-memory](../../decisions/agent/adr-0005-ai-correction-memory.md)

## 入口

| 类型 | 路径 |
|------|------|
| Rust（业务） | `apps/desktop/src-tauri/src/agent.rs`、`commands/agent.rs` |
| Gateway trait | `crates/ports/src/sidecar.rs` |
| Adapter | `crates/adapter/`（`RuntimeAgentSidecar`） |
| Runtime | `crates/runtime/`（`SidecarLifecycle`） |
| Python（例外 / 现有 ping 骨架） | `python/packages/gateway/`（`agent_ping` handler） |
| Contract | `contracts/`（`agent_ipc_ping_*` / `agent_sidecar_ping_*`） |

## 当前状态

PingAgent 骨架已实现，应用可通过 IPC 探活 Python sidecar。后续 AI 能力（LLM 推理、只读 Query 工具、纠错记忆等）默认按新 Change 在 Rust 实施；仅生态缺口才扩展 Python。
