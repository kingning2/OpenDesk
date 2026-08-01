# Contract Codegen

从 `contracts/schema/v1/` 生成两端类型：

```bash
pnpm contracts:sync
pnpm contracts:check
```

| 目标 | 目录 |
|---|---|
| Rust | `crates/common/src/contracts/` |
| TypeScript | `packages/contracts/src/generated/` |

实现脚本为 `skills/opendesk/scripts/sync-contracts.mjs`。生成文件带有标记，禁止手工修改。
