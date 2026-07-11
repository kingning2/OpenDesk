| Field | Value |
|-------|-------|
| ID | T001 |
| Priority | P0 |
| Status | completed |
| Depends on | — |
| Blocks | T002, T009, T011 |
| Milestone | M1 |

## Goal

消除 Skeleton 阶段违规项，使架构检查与 lint 全绿。

## Scope

- 移除 Tauri `greet` Demo
- 修复 `src-tauri/lib.rs` 的 `.expect()` 违规
- 前端根组件不再直连 `@tauri-apps/api`

## Out of scope

- 新业务功能
- 契约扩展

## Acceptance criteria

- [x] `check_architecture.py` 通过（无 unwrap/expect/panic）
- [x] `pnpm lint` 三端通过
- [x] `app.tsx` 无 `invoke("greet")`

## Key files

- `crates/app/src/lib.rs`
- `apps/desktop/src-tauri/src/lib.rs`
- `apps/desktop/src/app/app.tsx`
