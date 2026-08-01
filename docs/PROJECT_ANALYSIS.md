# OpenDesk 项目说明

OpenDesk 是本地优先的 AI 商务桌面，覆盖 YouTube 获客、客户管理、SMTP/IMAP 邮件与人工确认的 AI 辅助。

## 技术架构

```text
React → Tauri IPC → Rust
```

Rust 是唯一协调者与运行时，负责业务、SQLite、外部协议、Worker、Workflow Runtime 和模型调用；AI 基建位于 `crates/agent`。

跨端字段按 `Contract → codegen → Rust → React` 修改，生成 Rust 与 TypeScript 类型。

## 仓库

- `apps/desktop`：桌面 UI 与 Tauri 壳
- `crates`：Rust 业务与运行时
- `contracts`：跨端 Schema
- `docs/managed`：路线图、Domain、Change 与 ADR
- `skills/opendesk`：架构知识与 Node 工具

## 入口

- [工程 README](../README.md)
- [架构](architecture/README.md)
- [MVP 评审](managed/MVP_REVIEW.md)
- [开发知识库](../skills/opendesk/README.md)

```bash
pnpm lint
pnpm check:architecture
pnpm contracts:check
```
