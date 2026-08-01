# Rust / Tauri 规则

适用范围：`crates/**`、`apps/desktop/src-tauri/**`。

## 职责

- Rust 是唯一业务协调与运行时层。
- Tauri command 只做参数边界、调用应用层和错误映射。
- AI 基建集中在 `crates/agent`（`llm/`、`prompt/`、`skills/`）；工作流编排使用 `crates/workflow_runtime`；重任务使用 Worker 或明确的异步边界。
- SQLite、SMTP/IMAP、文件、网络与系统凭据都由 Rust 负责。

## 分层

- Domain 保持纯净；Application 编排 Port；Infrastructure 实现 IO；`crates/app` 负责装配。
- Feature 间不得依赖内部实现；跨域写传播用 Event，只读查询用 Query Port。
- Contract 类型来自 `crates/common/src/contracts/` 生成物。

## 质量

- 业务路径返回 `Result`，禁止 `unwrap`、`expect`、`panic!`。
- 异步或 CPU 密集任务不得阻塞 Tauri 主线程。
- 使用 `tracing` 并脱敏密钥、凭据和正文。
- 公开 API 写简洁中文 rustdoc；仅复杂多步骤函数用 `// 1.` 中文分段解释边界或原因。

验证：`pnpm lint:rust && pnpm check:architecture && pnpm contracts:check`。
