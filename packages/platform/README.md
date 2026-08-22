# @desk/platform

渠道 / 平台前端层 — 编译期平台选择、Tauri IPC 封装、事件订阅、错误分类、窗口封装。

## 边界

- **属于**：平台相关的桌面壳能力（IPC invoke 封装、Tauri 事件、编译期平台选择、
  错误分类、窗口能力）。
- **不属于**：UI 组件（`@desk/ui`）；业务状态（`@desk/store`）；契约类型
  （`@desk/contracts`，仅引用）。

## Usage

`apps/desktop` 通过 `@desk/platform/ipc/*`、`/compile`、`/events`、`/error` 子路径使用。
公开 API 由 `exports` 白名单约束；`src/ipc/chain.ts` 的虚拟模块
`virtual:dingda/platform-ipc-chain` 为包内私有，不对外。

## Directory

- `compile/` — 编译期平台选择（`__DINGDA_CHANNEL_PLATFORM__`）与静态路由常量
- `ipc/` — Tauri `invoke` 封装 + 每域 IPC 包装（account / item / order / license / …）
- `events/` — Tauri 事件订阅（channel / monitor / plugin / runtime）
- `error/` — 错误分类（network / ipc / code）+ 可注入 reporter
- `window/` — 窗口能力封装（唯一允许碰 `@tauri-apps/api/window`）
- `channel/` — 连接状态映射

## 禁止

- 在包内直接依赖 UI / store / 业务状态。
- 暴露 `ipc/chain`、`ipc/invoke`、`ipc/ipc-command-labels`、`ipc/shared`、`ipc/platforms/*`
  （内部实现，含虚拟模块）。
