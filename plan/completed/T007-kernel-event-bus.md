| Field | Value |
|-------|-------|
| ID | T007 |
| Priority | P1 |
| Status | completed |
| Depends on | T004 |
| Blocks | T008 |
| Milestone | M2 |

## Goal

将 `kernel::event` 从 no-op 升级为可用的进程内 Pub/Sub 骨架。

## Scope

- `EventBus` / `EventHandler` trait 保留
- `InMemoryEventBus`：topic 订阅表 + publish 分发
- 线程安全（`Mutex` / `RwLock`）
- 单元测试：publish → handler 收到 payload

## Out of scope

- 跨进程 / 持久化 event store
- Tauri Events 转发

## Acceptance criteria

- [x] `InMemoryEventBus::publish` 调用已注册 handler
- [x] 同一 topic 多 subscriber 均收到
- [x] handler 错误不 panic（返回 `EventError`）
- [x] 至少 1 个单元测试

## Key files

- `crates/kernel/src/event/mod.rs`
- `crates/kernel/Cargo.toml`
