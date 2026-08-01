# Rust Guide

Rust 是 OpenDesk 的唯一业务协调与运行时层。

## 组织

- `crates/app`：装配、命令与应用状态。
- Feature crate：Domain 与 Application。
- `crates/ports`：跨域/基础设施接口。
- Infrastructure crate：SQLite、邮件、Worker、系统集成。
- `crates/agent`：AI 基建（`llm/`、`prompt/`、`skills/`）；业务用例不放此 crate。
- `crates/workflow_runtime`：DAG、状态机、执行器、检查点与恢复。

## 约束

- Tauri command 保持薄，只做边界校验、调用和错误映射。
- 业务路径使用 `Result`，禁止 `unwrap`、`expect`、`panic!`。
- async IO 不阻塞线程；CPU/批量任务交给 Worker 或专用执行器。
- 公开 API 写简洁中文 rustdoc；复杂多步骤函数才用中文编号注释。
- 日志使用 `tracing` 并脱敏。

验证：`pnpm lint:rust && pnpm check:architecture`。
