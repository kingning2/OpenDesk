| Field | Value |
|-------|-------|
| ID | T017 |
| Priority | P1 |
| Status | completed |
| Depends on | T015, T016 |
| Blocks | — |
| Milestone | M5 |

## Goal

建立 Feature 页面骨架模板与占位路由，使 chat / mail / knowledge 等 Feature 有统一的「空页面」入口，agent 页面对齐同一模板。

## Scope

- **`@desk/ui` 或 `apps/desktop/src/app/`**
  - `PageScaffold`：标准页面结构（可选 subtitle + content 区 + 统一 padding）
  - `FeaturePlaceholderPage`：通用占位页（Feature 名 +「开发中」说明）
- **占位 Feature 路由**（仅前端骨架，无 IPC / 无 Rust crate）
  - `/features/chat`
  - `/features/mail`
  - `/features/knowledge`
  - 各 Feature 最小目录：`features/<name>/index.ts` + 占位 page
- **Agent 对齐**
  - `agent-page.tsx` 改用 `PageScaffold` 包裹现有 ping UI
- **文档**
  - 更新 `skills/opendesk/guides/frontend.md` 增加「布局与页面骨架」小节

## Out of scope

- 创建 chat/mail/knowledge 的 Contract / Rust crate / IPC
- 占位页内的真实业务 UI
- 国际化

## Acceptance criteria

- [x] 侧栏可导航至 chat / mail / knowledge 占位页
- [x] Agent 页使用与其他 Feature 一致的 `PageScaffold`
- [x] 占位 Feature 目录结构可作为 `add-feature.md` 的前端复制模板
- [x] `pnpm lint:frontend` 通过

## Key files

- `apps/desktop/src/app/pages/feature-placeholder.tsx`
- `apps/desktop/src/features/chat/`
- `apps/desktop/src/features/mail/`
- `apps/desktop/src/features/knowledge/`
- `apps/desktop/src/features/agent/agent-page.tsx`
- `apps/desktop/src/route/router.tsx`
- `skills/opendesk/guides/frontend.md`
