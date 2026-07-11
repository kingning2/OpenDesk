| Field | Value |
|-------|-------|
| ID | T002 |
| Priority | P0 |
| Status | completed |
| Depends on | T001 |
| Blocks | T003 |
| Milestone | M1 |

## Goal

建立 `contracts/` → 三端类型的 codegen 流水线（Skeleton 版）。

## Scope

- 增强 `skills/opendesk/scripts/sync_contracts.py`
- 新增 `contracts/codegen/README.md`
- 输出 TS / Rust / Python 到约定目录

## Out of scope

- OpenAPI → client 全自动生成
- 复杂 schema 特性（oneOf / allOf）

## Acceptance criteria

- [x] `sync_contracts.py` 可从 `schema/v1/` 生成类型
- [x] `packages/contracts/src/generated/` 有 index 导出
- [x] `crates/common/src/contracts/` 有对应 Rust struct
- [x] `python/packages/contracts/.../generated/` 有 TypedDict

## Key files

- `skills/opendesk/scripts/sync_contracts.py`
- `contracts/codegen/README.md`
- `packages/contracts/src/generated/`
- `crates/common/src/contracts/`
