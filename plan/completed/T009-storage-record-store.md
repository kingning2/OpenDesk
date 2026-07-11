| Field | Value |
|-------|-------|
| ID | T009 |
| Priority | P1 |
| Status | completed |
| Depends on | T001 |
| Blocks | — |
| Milestone | M2 |

## Goal

补全 `ports::RecordStore` 与 `storage::SqliteRecordStore` 的可替换 SQLite 骨架。

## Scope

- `ports::repository::RecordStore` 增加最小 CRUD trait 方法（空实现签名）
- `storage::repository::sqlite` 连接占位 + `ready()` 实现
- 禁止 UseCase 层直接 SQL

## Out of scope

- 真实表结构 / migration
- 具体 Feature repository

## Acceptance criteria

- [x] trait 定义在 `crates/ports`
- [x] SQLite 实现仅在 `crates/storage`
- [x] 可用内存 SQLite 或占位连接通过 `ready()`
- [x] 架构检查无跨层违规

## Key files

- `crates/ports/src/repository.rs`
- `crates/storage/src/repository/sqlite.rs`
