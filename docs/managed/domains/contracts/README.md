# Contracts Domain

`contracts/` 是 Rust 与 React 共享 DTO、IPC、Event 和 Error 的唯一真相源。

```text
Contract → pnpm contracts:sync → Rust → React
```

生成物：

- `crates/common/src/contracts/`
- `packages/contracts/src/generated/`

## 当前事实

`skills/opendesk/scripts/sync-contracts.mjs` 已生成两端类型；`pnpm contracts:check` 检查过期或多余生成物，`pnpm check:architecture` 会聚合该检查。

## 规则

- 生成文件禁止手改。
- Breaking Change 更新版本/迁移说明。
- `contracts/CHANGELOG.md` 只追加，不改历史条目。
