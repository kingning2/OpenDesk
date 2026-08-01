# OpenDesk Contracts

`contracts/` 是 Rust 与 React 共享 DTO、IPC、Event 和 Error 的唯一真相源。

## 变更顺序

```text
Contract → pnpm contracts:sync → Rust → React
```

生成目标：

- `crates/common/src/contracts/`
- `packages/contracts/src/generated/`

禁止手改生成物或先改实现再补 Schema。使用 `pnpm contracts:check` 验证同步状态。

## 目录

- `schema/v1/`：JSON Schema
- `codegen/`：codegen 说明
- `compatibility/`：字段规则与迁移指南
- `CHANGELOG.md`：只追加的契约历史

Breaking Change 应使用新版本或新文件，并更新 `compatibility/MIGRATION.md` 与 CHANGELOG。
