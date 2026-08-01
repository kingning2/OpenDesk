# Cursor Rules

- [`master.md`](master.md)：全仓库架构与质量基线
- [`branch-workflow.mdc`](branch-workflow.mdc)：分支命名与 scope
- [`active-branch.mdc`](active-branch.mdc)：`pnpm branch:sync` 生成的当前 scope
- [`frontend.md`](frontend.md)：React 与 UI
- [`rust.md`](rust.md)：Rust、Tauri、LLM 与后台任务

核心边界是 **React → Tauri IPC → Rust**；跨端变更顺序是 **Contract → codegen → Rust → React**。细节保持在对应规则或 [`skills/opendesk/`](../../skills/opendesk/) 中，避免重复。
