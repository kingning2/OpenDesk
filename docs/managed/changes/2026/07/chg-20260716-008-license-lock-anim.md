---
id: CHG-20260716-008-license-lock-anim
title: 激活结果锁裂开/加固动画
type: change
status: completed
priority: P1
owner: cursor-agent
domain: desktop-runtime
parent: T019
depends_on:
  - CHG-20260716-007-license-feedback-messages
blocks: []
milestone: unlock
created: 2026-07-16
updated: 2026-07-16
contracts: none
related:
  - docs/managed/changes/2026/07/chg-20260716-007-license-feedback-messages.md
---

# 激活结果锁裂开/加固动画

## 目标

激活校验结束后，锁图标播放结果动画：成功锁裂开后收起闸门；失败锁加固后回到可再试状态。校验过程中锁有轻微呼吸反馈。

## 非目标

- 改验签 / IPC
- 替换整套激活页布局
- 新增第三方动画库（复用 `@desk/ui` 的 `motion`）

## 背景

用户期望解锁反馈是「锁裂开 / 锁加固」的动画，而非仅 toast 文案。成功时若立刻 `onActivated` 卸载遮罩，动画看不到。

## 影响与边界

- 修改范围：`apps/desktop/src/features/license/**`、本 Change Record、`ACTIVE.md`
- 不修改范围：`python/**`、`crates/**`、`contracts/**`、`packages/ui/**`（仅消费已有 motion）
- Contract：无
- 跨层：否
- 跨 Feature：否
- 风险：成功路径需延迟卸载遮罩；需尊重 `prefers-reduced-motion`

## 依赖关系

- 父任务：T019
- 前置任务：CHG-20260716-007
- 阻塞任务：无

## 实施方案

1. 新增 `LicenseLockGlyph`：idle / busy / success（对半裂开）/ failure（加固脉冲）
2. Hook 暴露 `lockAnim`；成功播完动画再 `onActivated`；失败播完回 idle
3. Overlay 接入 glyph

## 验收

- [x] 成功可见锁裂开动画后再进主界面
- [x] 失败可见锁加固动画
- [x] 校验中有 busy 态反馈
- [x] tsc 通过
- [x] 实际结果已回填

## 实际结果

- 新增 `license-lock-glyph.tsx`：成功左右半锁外旋裂开 + 锁芯上移；失败描边加粗、外环加固、短震。
- `useLicenseActivate` 增加 `lockAnim`；成功等 ~900ms 再 `onActivated`；失败 ~700ms 回 idle；尊重 `prefers-reduced-motion`。
- Overlay 成功时收起表单，专注锁动画。
- 验证：`pnpm exec tsc --noEmit -p tsconfig.json`（apps/desktop）通过。

## 后续项

- 无
