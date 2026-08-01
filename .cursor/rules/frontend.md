---
description: OpenDesk React and UI boundaries
globs: apps/desktop/**/*.{ts,tsx},packages/ui/**/*.{ts,tsx},packages/platform/**/*.{ts,tsx},packages/store/**/*.{ts,tsx},packages/i18n/**/*.{ts,tsx}
alwaysApply: false
---

# Frontend 规则

## 边界

- React 只负责展示、交互和 UI 状态，通过 `@desk/platform/ipc` 调 Rust。
- Feature 禁止直接 `invoke()`、引入 `@tauri-apps/api`、访问 SQLite/文件或调用模型 API。
- 跨端类型来自 `packages/contracts/src/generated/`，禁止复制手写 DTO。

## 包职责

| 位置 | 负责 | 禁止 |
|---|---|---|
| `packages/ui` | 组件、令牌、主题、动效 | IPC、业务、Store |
| `packages/platform` | Tauri IPC 与 OS API 封装 | 业务规则 |
| `packages/store` | 共享 UI 状态底座 | IPC 结果与领域数据缓存 |
| `features/*` | 组合 UI、Feature hook、页面状态 | 其他 Feature 内部实现 |

IPC 数据放 Feature hook；局部状态用 `useState`，跨页面 UI 状态再用 Zustand。React Compiler 保持启用，不用无证据的手工 memo。

## UI

- Feature 使用 `@desk/ui` 语义组件，不直接引入 Radix/shadcn 源码，不堆裸 Tailwind 视觉类。
- 动效与交互修改前读取 `.cursor/skills/emil-design-eng/SKILL.md`；尊重 reduced motion。
- 文案经 `@desk/i18n`。

## 注释与验证

公开导出只在契约、边界或失败语义不明显时写简洁中文 JSDoc；注释解释原因，不复述 JSX。

验证：`pnpm lint:frontend && pnpm check:architecture && pnpm contracts:check`。
