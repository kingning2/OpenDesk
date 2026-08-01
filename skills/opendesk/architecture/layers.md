# 分层

## React

- Feature 页面与 hooks 组合 `@desk/ui`、`@desk/platform` 和生成 Contract。
- 只管理展示、交互与 UI 状态。
- 禁止直接 `invoke()`、数据库、文件 IO、模型调用和业务持久化。

## Tauri / Rust

- `apps/desktop/src-tauri`：注册 command、状态和事件。
- `crates/app`：应用组装与 IPC command 实现。
- Feature crate：Domain 与 Application 用例。
- `crates/ports`：跨域或基础设施 Port。
- Infrastructure crate：SQLite、SMTP/IMAP、Worker、系统能力。
- `crates/agent`：按 `llm/`、`prompt/` 等目录承载 AI 基础设施；不承载领域业务。
- `crates/workflow_runtime`：DAG、状态机、检查点、恢复与 Executor Registry。

Rust 内依赖方向：

```text
Tauri → Application → Domain
                    → Ports ← Infrastructure
```

Application 不直接写 SQL、HTTP 或文件；装配层注入实现。耗时 IO 使用 async，CPU/批量任务使用 Worker 或专用执行器。

## 通信

- React 请求：Tauri IPC。
- Rust 到 React 状态通知：Tauri Event。
- Rust Feature 写传播：`kernel::event`。
- 跨域只读：Query Port。
