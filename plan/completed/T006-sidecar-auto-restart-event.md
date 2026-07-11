| Field | Value |
|-------|-------|
| ID | T006 |
| Priority | P0 |
| Status | completed |
| Depends on | T005 |
| Blocks | — |
| Milestone | M2 |

## Goal

Sidecar 崩溃或 health 失败时自动重启，并通过 `kernel::event` 发布系统事件。

## Scope

- 契约：`runtime/event/sidecar.restarted.schema.json`
- `sync_contracts.py` 生成对应 DTO
- `SidecarLifecycle` 增加 watchdog / restart 策略（Skeleton：health 失败 → restart，限次）
- `InMemoryEventBus` 发布 `runtime.sidecar.restarted`（先用 T007 的 bus 或本任务内最小 impl）

## Out of scope

- 指数退避完整策略
- React 订阅系统事件

## Acceptance criteria

- [x] Contract event schema 存在且通过 `check_contracts.py`
- [x] sidecar 进程异常退出后 `ensure_running` 可重新拉起
- [x] 重启时发布 event（topic + payload 符合契约）
- [x] `pnpm lint` + `check_architecture.py` 通过

## Key files

- `contracts/schema/v1/runtime/event/`
- `crates/runtime/src/sidecar/lifecycle.rs`
- `crates/kernel/src/event/mod.rs`
