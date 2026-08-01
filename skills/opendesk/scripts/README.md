# OpenDesk Scripts

所有命令从仓库根目录运行。

| pnpm 命令 | 脚本 | 用途 |
|---|---|---|
| `pnpm branch` / `branch:create` / `branch:sync` | `branch-tools.mjs` | 创建分支并生成当前 scope |
| `pnpm contracts:sync` | `sync-contracts.mjs` | 从 JSON Schema 生成 Rust 与 TypeScript |
| `pnpm contracts:check` | `sync-contracts.mjs --check` | 检查生成物是否过期 |
| `pnpm check:architecture` | `check-architecture.mjs` | 检查 React/Rust 边界、零 Python 运行时、命名、Schema 与生成物 |

这些脚本是当前唯一受支持的 OpenDesk 仓库工具。新增工具前优先扩展根 `package.json` 的 pnpm 入口，避免文档直接绑定内部脚本参数。
