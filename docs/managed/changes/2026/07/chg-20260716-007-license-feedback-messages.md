---
id: CHG-20260716-007-license-feedback-messages
title: 激活校验操作补齐前端 message 提示
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

# 激活校验操作补齐前端 message 提示

## 目标

授权闸门校验与激活相关操作（状态检查、复制设备码、Token/Key 激活）在前端均有可见的 message / toast 反馈，用户不会在“无反应”状态下等待。

## 非目标

- 改 Rust 验签逻辑或 IPC 契约
- 重做激活页视觉设计
- 接入在线订阅或账号体系

## 背景

当前激活流程在 `busy` 时清空 `message`，按钮文案变化不明显；闸门初始校验仅有极淡遮罩；`@desk/ui` 已导出 `Toaster`/`toast` 但桌面端未挂载，成功后遮罩立刻卸载也看不到内联提示。

## 影响与边界

- 修改范围：`apps/desktop/src/app/app.tsx`、`apps/desktop/src/features/license/**`、本 Change Record
- 不修改范围：`python/**`、`crates/**`、`contracts/**`
- Contract：无
- 跨层：否（仅 React 反馈）
- 跨 Feature：否
- 风险：toast 与内联 message 可能重复，需文案一致、成功路径以 toast 保活

## 依赖关系

- 父任务：T019
- 前置任务：CHG-20260715-002
- 阻塞任务：无

## 实施方案

1. 在 `App` 挂载 `Toaster`；闸门 `loading` 显示“正在校验授权状态…”文案
2. 激活 Hook：操作开始即设 message + `toast.loading`，结束用 success/error 替换
3. 复制设备码、加载失败同样 toast；内联 message 在 busy 时显示“正在校验…”

## 验收

- [x] 闸门校验中可见文案提示
- [x] 复制 / 激活开始与结束均有 toast（或等价 message）
- [x] 激活失败可见错误文案；成功可见成功提示
- [x] lint / 类型检查通过
- [x] 实际结果已回填

## 实际结果

- `App` 挂载 `Toaster`；闸门 loading 展示「正在校验授权状态…」。
- `useLicenseActivate`：激活开始 `toast.loading("正在校验激活码…")` + 内联 message；结束 success/error；复制与设备码加载失败同样 toast。
- `useLicenseGate`：状态拉取失败 `toast.error`。
- 验证：`pnpm exec tsc --noEmit -p tsconfig.json`（apps/desktop）通过。

## 后续项

- 无
