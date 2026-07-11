| Field | Value |
|-------|-------|
| ID | T004 |
| Priority | P0 |
| Status | completed |
| Depends on | T003 |
| Blocks | T005, T006, T007, T010 |
| Milestone | M1 |

## Goal

Rust 接管 Python sidecar 生命周期：启动、健康检查、停止。

## Scope

- `runtime::sidecar::lifecycle` 模块
- spawn / health poll / stop on app exit
- `agent_ping` 前 `ensure_running()`
- 环境变量配置（port / dir / uv）

## Out of scope

- 崩溃自动重启（→ T006）
- 打包后 sidecar 路径探测

## Acceptance criteria

- [x] 应用启动自动 spawn sidecar（无需手动终端）
- [x] `GET /health` 轮询直到就绪或超时
- [x] `RunEvent::Exit` 时 stop sidecar
- [x] stdout/stderr pipe 到 Rust（原始行）

## Key files

- `crates/runtime/src/sidecar/lifecycle.rs`
- `crates/app/src/lib.rs`
- `python/sidecar/sidecar/`
