# Contracts Guide

新增或修改 DTO、IPC、Event、Error 字段时必须改 Contract。

```bash
pnpm contracts:sync
pnpm contracts:check
```

工作流：

1. 修改 `contracts/schema/v1/<feature>/...schema.json`。
2. 明确 required、可空、枚举、`additionalProperties` 与稳定 `$id`。
3. 生成 Rust 与 TypeScript。
4. 更新 Rust 实现，再更新 React。
5. 兼容性变化追加 CHANGELOG；Breaking Change 更新迁移说明。

禁止手改 `crates/common/src/contracts/` 与 `packages/contracts/src/generated/`。
