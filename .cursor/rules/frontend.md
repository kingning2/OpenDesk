# Developer A（Frontend）规则

适用范围：`apps/desktop/src/**`、`packages/ui/**`、`packages/platform/**`

## 职责边界

- UI 只负责展示与交互：**React → Rust（IPC）**，禁止直接调用 Python sidecar
- 禁止在 UI 中引入任何 Rust/Python 源码或路径
- 业务能力通过 `contracts/` 生成类型对齐，禁止手写 DTO（除临时原型且必须注明）

## 代码组织（必须）

- Feature 必须放在 `apps/desktop/src/features/{chat,mail,workflow,...}` 下
- 每个 feature 目录仅使用短名（一个词优先）
- IPC 调用必须通过 `packages/platform/src/ipc` 封装；组件内禁止直接 `@tauri-apps/api` 调用（后续会在 lint 中强制）

## 命名规范

- Feature：`chat`、`mail`、`workflow`（短名）
- Hook：`useChat`、`useMail`
- Store：`chatStore`、`userStore`

## 变更原则

- 所有跨端字段变更：先改 `contracts/`，再改 UI
- Breaking Change：必须走 `contracts/schema/v2`（或新文件）+ 迁移说明

