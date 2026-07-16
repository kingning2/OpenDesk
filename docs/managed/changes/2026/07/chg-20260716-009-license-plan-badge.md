---
id: CHG-20260716-009-license-plan-badge
title: 左下角展示套餐剩余时长
type: change
status: completed
priority: P1
owner: cursor-agent
domain: desktop-runtime
parent: T019
depends_on:
  - CHG-20260715-002-license-activation-gate
blocks: []
milestone: unlock
created: 2026-07-16
updated: 2026-07-16
contracts: none
related:
  - docs/managed/changes/2026/07/chg-20260715-002-license-activation-gate.md
---

# 左下角展示套餐剩余时长

## 目标

已激活且有 `expiresAt` 时，在主界面左下角持续展示套餐剩余时长；过期时显示已过期。

## 非目标

- 改 Rust / IPC 字段
- 设置页或详情弹窗
- 无锁开发构建展示假数据

## 影响与边界

- 修改范围：`apps/desktop/src/features/license/**`、`apps/desktop/src/app/**`
- 不修改范围：`crates/**`、`python/**`、`contracts/**`

## 验收

- [x] 有锁已激活时左下角显示剩余时长
- [x] 无锁 / 未激活 / 无 expiresAt 时不显示
- [x] tsc 通过

## 实际结果

- 新增 `LicensePlanBadge`（`fixed bottom-3 left-3`），读取 `LicenseStatus.expiresAt` 格式化为「剩余 X 天/小时/分钟」。
- `LicenseGateProvider` 共享 `useLicenseGate` 状态；`AppShell` 挂载角标。
- 不足 7 天 / 不足 1 天 / 已过期分别用 amber / urgent / destructive 样式。
- 验证：`pnpm exec tsc --noEmit -p tsconfig.json` 通过。

## 后续项

- 无
