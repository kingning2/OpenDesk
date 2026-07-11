| Field | Value |
|-------|-------|
| ID | T011 |
| Priority | P1 |
| Status | completed |
| Depends on | T001 |
| Blocks | T012 |
| Milestone | M3 |

## Goal

引入 react-router，建立 Feature 路由聚合骨架。

## Scope

- 安装 `react-router`（desktop app）
- `apps/desktop/src/route/index.ts` 导出路由表
- 根布局 + 占位 Feature 页（agent / chat 等入口）
- Sidebar 导航骨架（`@desk/ui` 组件）

## Out of scope

- 真实 Feature 业务 UI
- 跨 Feature 状态共享

## Acceptance criteria

- [x] `/` 与至少 1 个 `/features/agent` 路由可导航
- [x] Feature 层无 `@tauri-apps/api` 直连
- [x] `pnpm lint:frontend` 通过
- [x] 路由定义集中在 `route/`

## Key files

- `apps/desktop/src/route/`
- `apps/desktop/src/app/app.tsx`
- `apps/desktop/package.json`
