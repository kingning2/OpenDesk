---
id: CHG-20260720-001-desk-ui-form-primitives
title: 补齐 @desk/ui 表单原语并改回组件用法
type: change
status: completed
priority: P1
owner: cursor-agent
domain: frontend-ui
parent: none
depends_on: []
blocks: []
milestone: none
created: 2026-07-20
updated: 2026-07-20
contracts: none
related: []
---

# 补齐 @desk/ui 表单原语并改回组件用法

## 目标

在 `@desk/ui` 按现有 shadcn/new-york + Radix + CVA 技术栈补齐 `Button` / `Input` / `Select` / `ScrollArea`，并将 crawler / settings 页从原生控件改回组件实现。

## 非目标

- 不引入完整 shadcn CLI 批量生成。
- 不改 Contract / Rust / Python。
- 不做主题体系重构。

## 影响与边界

- 修改范围：`packages/ui/**`、`apps/desktop/src/features/crawler/**`、`apps/desktop/src/features/setting/**`
- Contract：无
- 跨层：否
- 跨 Feature：否

## 实际结果

- 新增 `@desk/ui` 原语：`Button` / `Input` / `Select*` / `ScrollArea`（Radix + CVA，对齐 shadcn new-york）
- `PageScaffold` 增加 `fill`；`CardHeader`/`CardContent` 支持 `compact` / `padding`
- crawler / settings 页改回组件用法
- 验证：`pnpm lint:types` 通过；相关文件 eslint 无 error

## 后续项

- 无
