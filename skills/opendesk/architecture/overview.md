# 架构总览

OpenDesk 使用两层桌面架构：

```mermaid
flowchart LR
  React[React Feature / UI] -->|@desk/platform IPC| Tauri[Tauri Commands]
  Tauri --> App[crates/app]
  App --> Feature[Rust Feature Crates]
  Feature --> Ports[Ports]
  Ports --> Infra[Storage / Mail / Worker]
  Feature --> Agent[crates/agent]
  Contracts[(contracts)] -. codegen .-> React
  Contracts -. codegen .-> Feature
```

## 职责

- React：展示、交互、UI 状态。
- Rust：业务编排、SQLite、SMTP/IMAP、Worker、Workflow Runtime、权限、日志与 AI。
- `crates/agent`：模型协议、Prompt 请求结构等无业务语义的 AI 基础设施。
- `contracts/`：Rust 与 TypeScript 的共享 DTO、IPC、Event 和 Error。

## 不变量

- React 只经 Tauri IPC 调 Rust。
- Feature 间只用 Contract、Event 或 Query Port。
- 跨端变更顺序为 `Contract → codegen → Rust → React`。
- 重任务不阻塞 Tauri 主线程。

细节见 [layers.md](layers.md)、[dependency.md](dependency.md)、[contracts.md](contracts.md)。
