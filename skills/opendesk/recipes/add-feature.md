# Recipe: Add Feature

1. 登记并批准 Change Record，确认分支 scope。
2. 搜索现有 Feature、Contract、Port 与 UI 组件。
3. 先在 `contracts/schema/v1/<feature>/` 定义跨端边界。
4. 运行 `pnpm contracts:sync`。
5. 在 Rust Feature/Application 实现用例，由 `crates/app` 与 Tauri 注册。
6. 在 `packages/platform` 封装 IPC，React Feature 组合 UI 与 hook。
7. 跨 Feature 写传播用 Event，只读查询用 Query Port。

验证：相关测试、`pnpm lint`、`pnpm check:architecture`、`pnpm contracts:check`。

模板见 [`../templates/feature/`](../templates/feature/) 与 [`../templates/ipc/`](../templates/ipc/)。
