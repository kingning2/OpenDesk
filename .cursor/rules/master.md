# Master（全局基线）规则

适用范围：**全仓库**（所有分支都必须遵守）。

## 1) 架构边界（必须）

- 固定三层：**React（展示） → Rust（Application Core） → Python（AI Runtime）**
- React 只通过 **Tauri IPC** 调 Rust；**禁止** React 以任何方式直连 Python sidecar（包括 `http://localhost:*` / WebSocket / SSE）
- Python 不知道 React；只提供对 Rust 的本机 IPC API（协议仍按 `contracts/` 定义）
- 数据库存储由 Rust 负责：**Python 禁止直连 SQLite/关系库**

### 1.1 Rust 必须掌控 Python Runtime（硬约束）

- Rust 必须负责 Python sidecar 的**生命周期管理**（启动/停止/重启/健康检查/超时）
- Rust 必须**接管 sidecar stdout/stderr**，并将其写入本地日志与可检索的结构化日志（按 `trace_id`/`task_id` 关联）
- Python sidecar 必须提供仅供 Rust 调用的**管理面接口**（建议：`/health`、`/stats`、`/tasks/active`、`/debug/dump`、`/metrics`），契约归档到 `contracts/openapi/sidecar.v1.yaml`
- Python sidecar 的业务输出（含流式 token）必须先回到 Rust，再由 Rust 通过 Tauri Events 转发给前端

## 2) 契约优先（必须）

- `contracts/` 是三端共享 **唯一真相源**（DTO / IPC / HTTP / Event / Error）
- 禁止三端各自维护 DTO（除临时原型，且必须在 PR 说明中标注并补契约）
- 任意跨端字段变更：**先改 contracts，再改实现**
- 破坏性变更（Breaking Change）：新增 `schema/v2`（或新文件）+ `contracts/compatibility/MIGRATION.md`

## 3) 模块通信原则（必须）

- 跨域/跨模块：优先 **Publish/Subscribe**（Rust `kernel::event`）
- 只读跨域查询：用 Query Port（trait），禁止互相 `use other_feature::*`
- 后台任务：统一走 `kernel::task`（Scheduler），禁止各模块私开长循环线程

## 4) 存储抽象（必须）

- 业务用例（UseCase）只依赖 `ports` trait（Repository/Store/Gateway）
- 禁止在业务用例中写 SQL/HTTP/文件 IO（这些只能在 `storage/vector/file/runtime` 实现层）
- 默认本地 SQLite，但要保证未来可替换（Postgres/MySQL/对象存储/向量库）

## 5) 命名规范（必须）

- 目录名/模块名/包名：短、行业通用、名词优先；**一个词优先，最多两个词**
- 禁止：`*Manager`、`*Service`、`*System`、`*Engine` 作为目录/crate 名
- 示例推荐：`chat`、`mail`、`agent`、`workflow`、`plugin`、`config`、`storage`、`runtime`、`kernel`

## 6) 三人协作边界（强约束）

- Developer A：`apps/desktop/src/**`、`packages/ui/**`、`packages/platform/**`
- Developer B：`crates/**`、`apps/desktop/src-tauri/**`
- Developer C：`python/**`
- 共同交叉点：`contracts/`（必须评审）

## 7) PR 与评审（必须）

- 合同（contracts）PR：至少 2 人 Approve，并更新 `CHANGELOG.md`
- 引入新模块/新命名：必须符合命名规范与目录约束
- 禁止在“未完成架构评审”阶段开始写业务功能（当前阶段只允许骨架、契约、规范与工具链）

## 8) 代码校验（必须）

提交前必须通过对应语言的 lint 检查；CI 也会强制执行。

| 语言 | 工具 | 命令 |
|------|------|------|
| Frontend | ESLint + TypeScript | `pnpm lint:frontend` |
| Rust | rustfmt + Clippy | `pnpm lint:rust` |
| Python | Ruff（check + format） | `pnpm lint:python` |
| 全量 | 三端一起 | `pnpm lint` |

### 8.1 Frontend（ESLint）

- 配置：`eslint.config.js`
- Feature UI（`apps/desktop/src/features/**`）**禁止**直接 import `@tauri-apps/api`，必须走 `@desk/platform/ipc`
- 自动修复：`pnpm lint:frontend:fix`

### 8.2 Rust（rustfmt + Clippy）

- 格式化：`rustfmt.toml`
- 静态分析：`clippy.toml` + `cargo clippy -D warnings`
- 快捷别名：`.cargo/config.toml`（`cargo lint`、`cargo fmt-check`）
- 自动格式化：`pnpm lint:rust:fix`

### 8.3 Python（Ruff）

- 配置：根目录 `pyproject.toml` 中 `[tool.ruff]`
- 检查 import 排序、常见 bug、现代化语法
- 自动修复：`pnpm lint:python:fix`

