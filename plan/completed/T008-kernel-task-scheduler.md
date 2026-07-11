| Field | Value |
|-------|-------|
| ID | T008 |
| Priority | P1 |
| Status | completed |
| Depends on | T007 |
| Blocks | — |
| Milestone | M2 |

## Goal

实现 `kernel::task` 后台任务调度骨架（延迟执行 + 取消）。

## Scope

- `TaskScheduler` trait 保留
- `InMemoryTaskScheduler`：tokio 延迟 spawn
- `schedule(name, delay_ms)` → `TaskId`
- `cancel(task_id)` 取消未执行任务

## Out of scope

- 持久化任务队列
- 分布式调度
- Python worker 集成

## Acceptance criteria

- [x] delay 到期后任务闭包执行
- [x] cancel 后任务不再执行
- [x] 无 unwrap/expect/panic
- [x] 至少 1 个单元测试

## Key files

- `crates/kernel/src/task/mod.rs`
- `crates/kernel/Cargo.toml`
