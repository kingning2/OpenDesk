---
description: OpenDesk frontend stack — React Compiler, shadcn/Radix, Tailwind, Motion, useState+Zustand, Feature hooks for IPC
globs: apps/desktop/**/*.{ts,tsx},packages/ui/**/*.{ts,tsx},packages/platform/**/*.{ts,tsx},packages/store/**/*.{ts,tsx},packages/i18n/**/*.{ts,tsx}
alwaysApply: false
---

# Frontend Rules（Developer A）

适用范围：`apps/desktop/src/**`、`packages/ui/**`、`packages/platform/**`、`packages/store/**`、`packages/i18n/**`

## 必须使用的技术栈


| 类别          | 库                         | 说明                                         |
| ----------- | ---------------------- | ------------------------------------------ |
| 编译          | **React Compiler**        | 必须启用，禁止仅靠手动 `useMemo`/`useCallback` 优化     |
| 组件基座        | **shadcn/ui + Radix UI**  | 组件源码在 `packages/ui`，可改不可绕                  |
| 本地 / 共享状态   | **useState + Zustand**    | 本地用 `useState`；跨页共享 UI 经 `@desk/store`；细则见 [react-state.mdc](react-state.mdc) |
| 服务器 / IPC 数据 | **Feature hooks**         | 经 `@desk/platform` 拉取并缓存在 Feature hook；**禁止**复制进 Zustand；**不强制** TanStack Query |
| 多语言        | **`@desk/i18n`**          | `createI18n`；文案字典放 app                       |
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
| `packages/store`    | Zustand 底座 · `createDeskStore`   | 业务领域状态 · IPC                               |
| `packages/i18n`     | `createI18n` · Provider / hooks    | 产品文案字典（放 app）                             |
| `packages/platform` | IPC · OS API                     | 业务逻辑                                        |
| `features/*`        | 组合 `@desk/ui` + `@desk/platform` + `@desk/store` + `@desk/i18n` | `@tauri-apps/api` · 裸 Tailwind · 直接引入 Radix |


## React Compiler

`apps/desktop/vite.config.ts` 必须启用 `babel-plugin-react-compiler`。

## IPC

Feature 只通过 `@desk/platform/ipc` 调 Rust，禁止 `invoke()` 与 `@tauri-apps/api`。

## 设计工程 Skill（强制）

前端 UI / 动效 / 交互打磨 **必须** 遵循 Emil Kowalski 设计工程 Skill（已安装）：

| Skill | 路径 | 何时使用 |
|-------|------|----------|
| **`emil-design-eng`** | [`.cursor/skills/emil-design-eng/SKILL.md`](../skills/emil-design-eng/SKILL.md) | 写或改 UI、组件、过渡、反馈态时 **先读再改** |
| `review-animations` | [`.cursor/skills/review-animations/SKILL.md`](../skills/review-animations/SKILL.md) | 动效 diff / PR 审查 |
| `animation-vocabulary` | [`.cursor/skills/animation-vocabulary/SKILL.md`](../skills/animation-vocabulary/SKILL.md) | 描述动效意图、写 prompt |
| `find-animation-opportunities` | [`.cursor/skills/find-animation-opportunities/SKILL.md`](../skills/find-animation-opportunities/SKILL.md) | 找可动效的交互点 |
| `improve-animations` | [`.cursor/skills/improve-animations/SKILL.md`](../skills/improve-animations/SKILL.md) | 改进现有动效 |
| `apple-design` | [`.cursor/skills/apple-design/SKILL.md`](../skills/apple-design/SKILL.md) | Apple HIG 对齐（与本仓库 Apple 风令牌互补） |

硬约束摘要（细节以 `emil-design-eng` 全文为准）：

- UI 动效通常 **低于 300ms**；优先 `ease-out` / 自定义曲线，避免 `ease-in`
- 只动画 **`transform` / `opacity`**；禁止无故 `transition: all`
- 不要从 `scale(0)` 出现；popover 从 trigger 原点缩放
- 高频操作（键盘、百次/日）少动或不动；尊重 `prefers-reduced-motion`
- 动效审查输出 **Before / After / Why** 表格

安装来源：`npx skills add emilkowalski/skill`（`skills-lock.json`）；Cursor 副本在 `.cursor/skills/`。

**自检命令：** 在 Agent 对话输入 `/check-emil-design`（可跟路径或「当前 diff」），按 Emil 标准审查 UI/动效并给出 `PASS` / `FAIL`。

## 相关文档

- `packages/ui/README.md`
- `packages/store/README.md`
- `skills/opendesk/guides/ui-design-system.md`
- `.cursor/skills/emil-design-eng/SKILL.md`

## 补充职责边界

- UI 只负责展示与交互：**React -> Rust（IPC）**，禁止直接调用 Python sidecar
- 禁止在 UI 中引入任何 Rust/Python 源码或路径
- 业务能力通过 `contracts/` 生成类型对齐，禁止手写 DTO（除临时原型且必须注明）

## 代码组织（必须）

- Feature 必须放在 `apps/desktop/src/features/{chat,mail,workflow,...}` 下
- 每个 feature 目录仅使用短名（一个词优先）
- IPC 调用必须通过 `packages/platform/src/ipc` 封装；组件内禁止直接 `@tauri-apps/api` 调用
- 客户端**共享 UI**状态经 `@desk/store`（Zustand）；业务 store 定义在 Feature / 壳层，禁止把领域列表/IPC 结果塞进 `packages/store`
- 状态分类与自检见 [react-state.mdc](react-state.mdc)（本地 `useState`、共享 Zustand、IPC 用 Feature hook；不强制 TanStack Query）

## 命名规范

- Feature：`chat`、`mail`、`workflow`
- Hook：`useChat`、`useMail`
- Store：`chatStore`、`userStore`

## 变更原则

- 所有跨端字段变更：先改 `contracts/`，再改 UI
- Breaking Change：必须走 `contracts/schema/v2`（或新文件）+ 迁移说明
