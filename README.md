# OpenDesk

本地优先的 AI 商务桌面：YouTube 获客、SMTP/IMAP 邮件谈价与人工确认的 AI 辅助。

## 架构

```text
React → Tauri IPC → Rust
```

Rust 是唯一协调者与运行时，负责业务、SQLite、任务、外部服务和 AI；AI 基建位于 `crates/agent`（`llm/`、`prompt/`、`skills/`），业务 Prompt/规则留在所属 Feature。React 不直连数据库或模型服务。

跨端类型以 `contracts/` 为唯一真相源：

```text
Contract → codegen → Rust → React
```

## 目录

- `apps/desktop`：Tauri + React 桌面应用
- `packages`：UI、平台 IPC 与生成的 TypeScript 契约
- `crates`：Rust workspace，含业务、存储、Worker、Workflow Runtime 与 `llm`
- `contracts`：JSON Schema 契约
- `docs/managed`：路线图、Domain、Change 与 ADR
- `skills/opendesk`：开发知识库与 Node 工具

## 开发与检查

```bash
pnpm install
pnpm tauri dev
pnpm lint
pnpm check:architecture
pnpm contracts:check
```

契约变化后运行 `pnpm contracts:sync`。切换分支后运行 `pnpm branch:sync`。

## 文档入口

- [架构规则](.cursor/rules/master.md)
- [产品与技术架构](docs/architecture/README.md)
- [Managed Docs](docs/managed/README.md)
- [MVP 路线图](docs/managed/roadmaps/mvp-sales-workbench.md)
- [OpenDesk Skill](skills/opendesk/README.md)
