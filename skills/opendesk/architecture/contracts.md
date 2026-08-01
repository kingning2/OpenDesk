# Contracts

`contracts/schema/v1/` 是 Rust 与 React 共享 DTO、IPC、Event 和 Error 的唯一真相源。

## 流程

```text
修改 Schema
  → pnpm contracts:sync
  → crates/common/src/contracts/
  → packages/contracts/src/generated/
  → Rust 实现
  → React 使用
```

`pnpm contracts:check` 检查生成物是否同步；生成文件禁止手改。

## 兼容

- 新增可选字段可保留当前版本。
- 删除、重命名或新增必填字段需评估新版本并更新迁移说明。
- 更新 `contracts/CHANGELOG.md` 时只追加，不改写历史条目。
- JSON Schema 必须有稳定 `$id`、明确 required 和 `additionalProperties` 策略。

详见 [`../guides/contracts.md`](../guides/contracts.md) 与 [`../../../contracts/README.md`](../../../contracts/README.md)。
