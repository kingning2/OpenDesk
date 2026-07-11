---
description: OpenDesk frontend stack — React Compiler, shadcn/Radix, Tailwind, Motion, Apple-style design tokens in @desk/ui
globs: apps/desktop/**/*.{ts,tsx},packages/ui/**/*.{ts,tsx},packages/platform/**/*.{ts,tsx}
alwaysApply: false
---

# Frontend Rules（Developer A）

适用范围：`apps/desktop/src/**`、`packages/ui/**`、`packages/platform/**`

## 必须使用的技术栈


| 类别          | 库                         | 说明                                         |
| ----------- | ------------------------- | ------------------------------------------ |
| 编译          | **React Compiler**        | 必须启用，禁止仅靠手动 `useMemo`/`useCallback` 优化     |
| 组件基座        | **shadcn/ui + Radix UI**  | 组件源码在 `packages/ui`，可改不可绕                  |
| 样式          | **Tailwind CSS**          | 仅允许在 `packages/ui` 内写 utility；Feature 禁止裸用 |
| 动画          | **Motion**                | Spring / 过渡；禁止 Feature 直接写 CSS animation   |
| 图标          | **Lucide React**          | 统一 `@desk/ui/icons` 导出                     |
| 表格          | **TanStack Table**        | 经 `@desk/ui` Table 封装                      |
| 虚拟列表        | **TanStack Virtual**      | 经 `@desk/ui` VirtualList 封装                |
| 表单          | **Zod + React Hook Form** | 经 `@desk/ui` Form 封装                       |
| 日期          | **date-fns**              | 经 `@desk/ui` 日期组件封装                        |
| 主题          | **next-themes**           | 经 `@desk/ui` ThemeProvider                 |
| 拖拽          | **dnd-kit**               | 经 `@desk/ui` 拖拽 primitives                 |
| 命令面板        | **cmdk**                  | 经 `@desk/ui` Command 封装                    |
| Toast       | **Sonner**                | 经 `@desk/ui` Toaster                       |
| Workflow 编辑 | **Monaco Editor**         | 经 `@desk/ui` WorkflowEditor 封装             |


## Apple 风设计令牌（`packages/ui`）

必须在 `packages/ui/src/tokens/` 定义并导出：

- **Glass** — 半透明材质
- **Blur / Backdrop** — 背景模糊
- **Motion / Spring** — 动画曲线预设
- **Dynamic Color** — 语义色（非硬编码 hex）
- **Radius** — 圆角刻度
- **Typography** — 字体阶梯

Feature **禁止**直接使用 `bg-white`、`rounded-lg` 等原子类。

```tsx
// ❌ Feature 中禁止
<div className="bg-white rounded-lg shadow-md backdrop-blur-xl" />

// ✅ 使用 @desk/ui 语义组件
import { Card } from "@desk/ui";
<Card variant="glass">...</Card>
```

## 分层边界


| 包                   | 允许                               | 禁止                                          |
| ------------------- | -------------------------------- | ------------------------------------------- |
| `packages/ui`       | 组件 · 令牌 · 主题 · 动画                | IPC · 业务 · Store · API                      |
| `packages/platform` | IPC · OS API                     | 业务逻辑                                        |
| `features/*`        | 组合 `@desk/ui` + `@desk/platform` | `@tauri-apps/api` · 裸 Tailwind · 直接引入 Radix |


## React Compiler

`apps/desktop/vite.config.ts` 必须启用 `babel-plugin-react-compiler`。

## IPC

Feature 只通过 `@desk/platform/ipc` 调 Rust，禁止 `invoke()` 与 `@tauri-apps/api`。

## 相关文档

- `packages/ui/README.md`
- `skills/opendesk/guides/ui-design-system.md`

## 补充职责边界

- UI 只负责展示与交互：**React -> Rust（IPC）**，禁止直接调用 Python sidecar
- 禁止在 UI 中引入任何 Rust/Python 源码或路径
- 业务能力通过 `contracts/` 生成类型对齐，禁止手写 DTO（除临时原型且必须注明）

## 代码组织（必须）

- Feature 必须放在 `apps/desktop/src/features/{chat,mail,workflow,...}` 下
- 每个 feature 目录仅使用短名（一个词优先）
- IPC 调用必须通过 `packages/platform/src/ipc` 封装；组件内禁止直接 `@tauri-apps/api` 调用

## 命名规范

- Feature：`chat`、`mail`、`workflow`
- Hook：`useChat`、`useMail`
- Store：`chatStore`、`userStore`

## 变更原则

- 所有跨端字段变更：先改 `contracts/`，再改 UI
- Breaking Change：必须走 `contracts/schema/v2`（或新文件）+ 迁移说明

