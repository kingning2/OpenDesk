---
name: opendesk
description: OpenDesk 本地优先 AI 商务桌面开发知识库。用于仓库内功能、Contract、IPC、Rust crate、架构与 review；约束 React→Tauri IPC→Rust、Rust 内置 LLM、Contract-first 和 Feature 边界。
---

# OpenDesk Development Skill

## 开始

1. 读取 [`skills/opendesk/README.md`](../../../skills/opendesk/README.md)。
2. 按任务读取一个相关 Recipe 或 Guide。
3. 先搜索现有 Contract、crate、Feature、模板和命令，再决定是否新增。
4. 遵守 Managed Docs Change 门禁和当前分支 scope。

## 不变量

```text
React → Tauri IPC → Rust
Contract → codegen → Rust → React
```

- Rust 是唯一协调者与运行时；AI 基建在 `crates/agent`（`llm/`、`prompt/`、`skills/`），业务 Prompt 留在 Feature。
- React 不直连数据库、文件系统或模型服务。
- Feature 间只通过 Contract、Event 或 Query Port 协作。
- 生成的 Rust/TypeScript Contract 不手改。

## 常用入口

| 任务 | 文档 |
|---|---|
| 总体架构 | `architecture/overview.md` |
| IPC / Contract | `guides/ipc.md`、`guides/contracts.md` |
| Rust / React | `guides/rust.md`、`guides/frontend.md` |
| Feature / crate / workflow | `recipes/add-feature.md`、`add-crate.md`、`add-workflow.md` |
| Agent / Provider | `recipes/add-agent.md`、`add-provider.md` |
| Review | `guides/review.md` |

## 验证

```bash
pnpm lint
pnpm check:architecture
pnpm contracts:check
```

契约变化先运行 `pnpm contracts:sync`。
