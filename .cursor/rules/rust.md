# Developer B（Rust）规则

适用范围：`crates/**`、`apps/desktop/src-tauri/**`

## 职责边界

- Rust 是桌面端 **Application Core / 协调中枢**
- React 只能通过 Tauri IPC 调 Rust；Python 只能通过 Rust `runtime` 访问
- SQLite / 存储层由 Rust 独占（Python 不直连 DB）

## 分层与依赖（必须）

- Feature crate（如 `crates/chat`）内部按 `domain/` 与 `app/` 分层
- `domain/` 禁止 IO（无 SQL/HTTP/文件/tauri）
- `app/` 只依赖 `ports` trait，不允许依赖 `storage/vector/file/runtime` 的具体实现
- `kernel` 只提供横切能力（event/task/config/logger/plugin），禁止写具体业务规则

## 通信模式

- 跨 feature：优先 `kernel::event` Pub/Sub，禁止直接 `use other_feature::*`
- 后台任务：必须通过 `kernel::task` 注册/调度，禁止私开长循环线程

## 命名规范

- crate 名必须为短名名词：`chat`、`mail`、`kernel`、`storage`
- 禁止：`chat-service`、`workflow-manager`、`*_engine`

## 契约变更

- 所有跨端 DTO/事件/错误码：必须来源 `contracts/`
- Breaking Change：新 `schema/v2` 或新文件 + 迁移说明

