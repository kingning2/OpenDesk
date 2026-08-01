# OpenDesk 开发知识库

## 架构

```text
React（apps/desktop、packages/*）
  → Tauri IPC（@desk/platform/ipc）
Rust（crates/*、src-tauri）
```

Rust 是唯一协调者与运行时：业务、存储、任务、外部协议和 AI 都在 Rust 内完成；AI 基建集中于 `crates/agent`（`llm/`、`prompt/`、`skills/`），业务用例不放此 crate。

跨端顺序固定为：

```text
contracts/ → pnpm contracts:sync → Rust → React
```

生成物位于 `crates/common/src/contracts/` 与 `packages/contracts/src/generated/`。

## 使用方式

1. 从 [`architecture/overview.md`](architecture/overview.md) 确认边界。
2. 只读取一个匹配的 [`guides/`](guides/) 或 [`recipes/`](recipes/)。
3. 复用 [`templates/`](templates/) 中仍适用的骨架。
4. 使用 [`scripts/`](scripts/) 中的 Node 工具和根级 pnpm 命令。

## 常用命令

```bash
pnpm branch:sync
pnpm contracts:sync
pnpm contracts:check
pnpm check:architecture
pnpm lint
```

## 目录

- `architecture/`：稳定架构事实
- `guides/`：IPC、Contract、Rust、React、日志、测试等规范
- `recipes/`：新增能力的最短步骤
- `templates/`：可复制骨架
- `scripts/`：`branch-tools.mjs`、`sync-contracts.mjs`、`check-architecture.mjs`
- `examples/`：Contract、Event、React 与 Rust 示例

修改前还需遵守根 [`AGENTS.md`](../../AGENTS.md) 的 Managed Docs 门禁与当前分支 scope。
