| Field | Value |
|-------|-------|
| ID | T003 |
| Priority | P0 |
| Status | completed |
| Depends on | T002 |
| Blocks | T004, T012 |
| Milestone | M1 |

## Goal

打通首条端到端链路：`agent/ping`（React → Rust → Python）。

## Scope

- 契约：`agent/sidecar/ping.*` + `agent/ipc/ping.*`
- Rust：`SidecarClient` + `agent_ping` route + Tauri command
- Python：gateway handler + sidecar 路由注册
- React：`@desk/platform/ipc/agent` + 骨架页按钮

## Out of scope

- 真实 agent 业务逻辑
- 流式输出

## Acceptance criteria

- [x] Contract 成对存在且 `check_contracts.py` 通过
- [x] Rust `runtime::sidecar::routes::agent_ping` 可 POST
- [x] Python handler 返回 `{ok, trace_id}`
- [x] Tauri `agent_ping` command 注册
- [x] 前端经 `@desk/platform/ipc` 调用（非直连 Tauri）

## Key files

- `contracts/schema/v1/agent/`
- `crates/runtime/src/sidecar/`
- `crates/agent/src/app/ping.rs`
- `python/packages/gateway/src/gateway/handlers/agent_ping.py`
- `packages/platform/src/ipc/agent.ts`
