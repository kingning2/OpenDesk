# OpenDesk 全局规则

适用范围：全仓库。

## 架构边界

```text
React（展示） → Tauri IPC → Rust（唯一协调者与运行时）
```

| 允许 | 禁止 |
|---|---|
| Feature 通过 `@desk/platform/ipc` 调 Rust | Feature 直接 `invoke()` 或直连外部运行时 |
| Rust 访问 SQLite、网络、文件和模型服务 | React 访问 SQLite、文件系统或模型 API |
| Rust 通过 `crates/agent`（`llm/`）调用 LLM | 在 React 中实现 AI 或业务规则 |
| Rust 通过 Tauri Event 通知 React | 后台进程直接操作 WebView |

## Contract First

`contracts/` 是 Rust 与 React 共享 DTO、IPC、Event 和 Error 的唯一真相源。

```text
Contract → pnpm contracts:sync → Rust → React
```

- 生成物位于 `crates/common/src/contracts/` 与 `packages/contracts/src/generated/`，禁止手改。
- Breaking Change 使用新版本并更新迁移说明；历史 `contracts/CHANGELOG.md` 条目不得改写。

## Rust 与 Feature 边界

- Rust 内按 Domain/Application/Port/Infrastructure 分责；业务用例不直接写 SQL、HTTP、文件或 Tauri 调用。
- Feature 间禁止直接调用内部实现。写传播用 `kernel::event`，只读跨域查询用 Query Port，共享数据用 Contract。
- `crates/app` 与 Tauri 壳负责组装；`crates/agent` 只提供 AI 基建（`llm/`、`prompt/`、`skills/`），业务 Prompt/用例留在 Feature；长任务由 Workflow Runtime、Worker 或明确的异步任务承载。
- 业务路径使用 `Result` 与明确错误类型，禁止 `unwrap`、`expect`、`panic!`。
- 所有后端操作（command / UseCase / worker handler / 后台任务）必须在应用边界用 `tracing` 记录：操作名、实体/任务 id、**开始与结果**；长任务另记录阶段/进度（低频，避免循环内噪声）。不得记录密钥或完整敏感正文。具体清单见 `skills/opendesk/guides/logging.md`。

## React 边界

- `packages/ui` 只含组件、令牌、主题与动效；`packages/platform` 封装 IPC/OS 能力；Feature 组合两者。
- IPC 数据保留在 Feature hook；共享 UI 状态可用 Zustand，不把服务端/IPC 结果复制为全局业务 store。
- Feature 不直接引入 `@tauri-apps/api`、Radix 源码或裸 Tailwind 视觉实现。
- UI 与动效遵循 `.cursor/skills/emil-design-eng/SKILL.md`。

## 代码与文档

- 公开 API 使用简洁中文 rustdoc/JSDoc，说明用途、约束与失败边界；不写作者、日期或机械参数模板。
- 仅复杂多步骤函数使用 `// 1.`、`// 2.` 中文分段，注释解释为什么或边界，禁止复述代码。
- 修改前遵守 `docs/managed/` 的 Change 门禁；历史 Change/ADR 正文只读。

## 分支与验证

分支 scope 由 `skills/opendesk/config/branch_roles.json` 生成：

```bash
pnpm branch:sync
pnpm lint
pnpm check:architecture
pnpm contracts:check
```

完整分支流程见 [`branch-workflow.mdc`](branch-workflow.mdc)。
