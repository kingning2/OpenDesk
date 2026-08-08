# OpenDesk

本地优先的 AI 商务桌面。当前运行时只有两层：

```text
React（展示） → Tauri IPC → Rust（业务、存储、任务与 AI）
```

- React 只通过 `@desk/platform/ipc` 调用 Rust。
- Rust 是唯一协调者与运行时；AI 基建位于 `crates/agent`（`llm/`、`prompt/`、`skills/`），业务 Prompt 留在所属 Feature。
- 分层：`crates/` 只放基建（`common`/`ports`/`storage`/`kernel`/`adapter`/`agent`/`mail-net`）；业务代码（`chat`/`customer`/`mail`/`crawler`/`knowledge`/`workflow_runtime`/`worker`/`app-core`/`opendesk-skills`）位于 `business/`，桌面（`apps/desktop/src-tauri/crates/app`）与 web（`apps/web/src-axum`）共用。
- `contracts/` 是跨端唯一真相源，顺序固定为 **Contract → codegen → Rust → React**。
- Feature 间禁止直接依赖；只使用 Contract、Event 或 Query Port。

完整规则见 [`.cursor/rules/master.md`](.cursor/rules/master.md)，开发知识库见 [`skills/opendesk/`](skills/opendesk/)。

## 分支范围

分支名格式为 `<role>/<kind>/<slug>`：

- `frontend/*/*`：桌面端 React、Tauri 与 Rust；
- `contract/*/*`：契约与两端生成物；
- `main`：集成；
- 其他分支名按集成范围处理。

```bash
pnpm branch:create frontend feature mail-sync
pnpm branch:create contract chore mail-schema
pnpm branch:sync
```

配置源为 [`skills/opendesk/config/branch_roles.json`](skills/opendesk/config/branch_roles.json)；切换分支后必须运行 `pnpm branch:sync`。

## Managed Docs 门禁

处理仓库改动时按需读取：

1. [`docs/managed/README.md`](docs/managed/README.md)；
2. [`docs/managed/registry/ACTIVE.md`](docs/managed/registry/ACTIVE.md)；
3. 与改动路径匹配的 Domain；
4. 当前 Change Record；
5. 仅在冲突时读取相关 ADR 或历史 Change。

代码、契约、配置或依赖修改前必须登记并批准 Change Record；完成后由负责人回填结果、移出 ACTIVE，并按需更新 Domain。不要修改历史 Change/ADR 正文。纯只读分析无需登记。

## 最小质量门

```bash
pnpm lint
pnpm check:architecture
pnpm contracts:check
```

Review 时检查：分支 scope、Contract-first、React/Rust 边界、Feature 边界、错误处理与验证结果。
