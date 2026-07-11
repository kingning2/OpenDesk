| Field | Value |
|-------|-------|
| ID | T012 |
| Priority | P1 |
| Status | completed |
| Depends on | T003, T011 |
| Blocks | — |
| Milestone | M3 |

## Goal

将 agent Feature 做成可复制的垂直切片模板（契约 → IPC → 页面）。

## Scope

- `apps/desktop/src/features/agent/` 页面 + hook
- hook 调用 `@desk/platform/ipc/agent`
- Feature `index.ts` 导出注册项
- 文档：`skills/opendesk/examples/` 或 feature README 说明复制步骤

## Out of scope

- LLM / 真实 agent 推理
- 跨 Feature 调用

## Acceptance criteria

- [x] agent Feature 页可触发 ping 并展示结果
- [x] 代码仅在 `features/agent/` + platform ipc（无 Tauri 直连）
- [x] 可作为 chat/mail 等 Feature 的复制模板
- [x] `check_layers.py` 通过

## Key files

- `apps/desktop/src/features/agent/`
- `packages/platform/src/ipc/agent.ts`
- `skills/opendesk/recipes/add-feature.md`
