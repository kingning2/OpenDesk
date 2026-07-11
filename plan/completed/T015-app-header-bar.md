| Field | Value |
|-------|-------|
| ID | T015 |
| Priority | P1 |
| Status | completed |
| Depends on | T013, T014 |
| Blocks | T016, T017 |
| Milestone | M5 |

## Goal

在自定义标题栏下方增加应用级 Header（页面标题、面包屑占位、主题切换、操作区插槽），形成完整的「头部」体验。

## Scope

- **`@desk/ui`**
  - `AppHeader` 组件：`title`、`description`（可选）、`actions`（ReactNode 插槽）
  - `ThemeToggle` 组件：基于现有 `ThemeProvider`（next-themes）切换 light/dark/system
- **`apps/desktop`**
  - 路由 → 标题映射（`route/meta` 或轻量 `getPageMeta(pathname)` 工具）
  - Shell 在 `MainPanel` 顶部渲染 `AppHeader`，随路由变化更新标题
  - Home / Agent 等已有路由接入标题

## Out of scope

- 真实面包屑多级导航（仅占位 prop）
- 用户头像 / 通知中心等业务控件
- Command Palette（⌘K，后续独立任务）

## Acceptance criteria

- [x] 切换路由时 Header 标题同步更新
- [x] 主题切换可用且持久化（next-themes 默认行为）
- [x] Header 与 TitleBar 视觉分层清晰（TitleBar 窗口级，AppHeader 页面级）
- [x] Feature 层无裸 Tailwind，通过 `@desk/ui` 组合
- [x] `pnpm lint:frontend` 通过

## Key files

- `packages/ui/src/components/layout/app-header.tsx`
- `packages/ui/src/components/theme-toggle.tsx`
- `apps/desktop/src/app/shell.tsx`
- `apps/desktop/src/route/page-meta.ts`（或等价文件）
