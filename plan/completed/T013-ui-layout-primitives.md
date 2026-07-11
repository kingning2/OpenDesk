| Field | Value |
|-------|-------|
| ID | T013 |
| Priority | P0 |
| Status | completed |
| Depends on | T011 |
| Blocks | T014, T015, T016 |
| Milestone | M5 |

## Goal

在 `@desk/ui` 建立 App 级布局原语，供 Shell 与 Feature 页面复用，替代 shell 内联 Tailwind 拼装。

## Scope

- `packages/ui/src/components/layout/`：
  - `AppLayout` — 根 flex 容器（sidebar + main 列）
  - `Sidebar` — 固定宽度侧栏容器（glass variant）
  - `SidebarNav` / `SidebarNavItem` — 导航列表与链接槽
  - `MainPanel` — 主内容区（含 header 槽 + content 槽）
  - `PageContainer` — 页面内容最大宽度与内边距
- 从 `@desk/ui` 统一导出
- 仅骨架与 variant，无业务状态

## Out of scope

- Tauri 窗口控制
- Feature 业务 UI
- 响应式折叠侧栏（后续任务）

## Acceptance criteria

- [x] 五个布局组件可在 Storybook 式占位页或 shell 中组合渲染
- [x] Feature 层可通过 `@desk/ui` 导入，无需裸 Tailwind 原子类
- [x] 使用 `tokens/` 中的 glass / radius / typography 令牌
- [x] `pnpm lint:frontend` 通过

## Key files

- `packages/ui/src/components/layout/`
- `packages/ui/src/index.ts`
