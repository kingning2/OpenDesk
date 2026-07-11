| Field | Value |
|-------|-------|
| ID | T016 |
| Priority | P1 |
| Status | pending |
| Depends on | T013, T014, T015 |
| Blocks | T017 |
| Milestone | M5 |

## Goal

将 `AppShell` 重构为基于 `@desk/ui` 布局原语的组装层，并建立 Feature 导航注册表，便于后续 Feature 自助挂载侧栏入口。

## Scope

- **Shell 重构**
  - `apps/desktop/src/app/shell.tsx`：组合 `TitleBar` + `AppLayout` + `Sidebar` + `AppHeader` + `Outlet`
  - 移除 shell 内联 Tailwind 导航样式，改用 `SidebarNav` / `SidebarNavItem`
- **导航注册表**
  - `apps/desktop/src/route/nav-registry.ts`：聚合各 Feature 导出的 `{ id, path, label, icon? }`
  - 各 Feature `index.ts` 扩展 `navItem` 字段（agent 先行，作为模板）
- **路由**
  - `router.tsx` 保持集中定义；新增 Feature 路由时同步注册 nav

## Out of scope

- 动态 Feature 插件加载
- 侧栏折叠 / 拖拽调宽
- 权限过滤导航项

## Acceptance criteria

- [ ] Shell 文件仅负责组装，不含复杂样式逻辑
- [ ] 新增 Feature 只需：export navItem + 在 registry 聚合 + router 加路由
- [ ] 现有 Home / Agent 导航行为与 T011 一致
- [ ] `check_layers.py` 与 `pnpm lint:frontend` 通过

## Key files

- `apps/desktop/src/app/shell.tsx`
- `apps/desktop/src/route/nav-registry.ts`
- `apps/desktop/src/route/router.tsx`
- `apps/desktop/src/features/agent/index.ts`
