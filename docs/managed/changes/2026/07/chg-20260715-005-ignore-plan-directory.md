---
id: CHG-20260715-005-ignore-plan-directory
title: 忽略根目录 plan 文件夹
type: change
status: completed
priority: P2
owner: codex
domain: documentation
parent: none
depends_on: []
blocks: []
milestone: none
created: 2026-07-15
updated: 2026-07-15
contracts: none
related: []
---

# 忽略根目录 plan 文件夹

## 目标

让 Git 忽略仓库根目录的 `plan/` 文件夹及其内容。

## 非目标

- 不删除 `plan/` 中的文件。
- 不处理已被 Git 跟踪的文件。

## 背景

用户明确要求将 `plan` 文件夹加入 `.gitignore`，本次请求视为已批准实施。

## 影响与边界

- 修改范围：根 `.gitignore`、本 Change Record 与 Active Registry。
- 不修改范围：业务代码、Contract、依赖和其他忽略规则。
- Contract：无。
- 跨层：否。
- 跨 Feature：否。
- 风险：若 `plan/` 中已有跟踪文件，仅新增忽略规则不会自动取消跟踪。

## 依赖关系

- 父任务：无。
- 前置任务：无。
- 阻塞任务：无。

## 实施方案

1. 在根 `.gitignore` 添加仅匹配根目录 `plan/` 的规则。
2. 使用 `git check-ignore` 验证规则。

## 验收

- [x] `plan/` 下未跟踪文件被忽略
- [x] 其他路径不受影响
- [x] 架构边界检查通过
- [x] 实际结果已回填

## 实际结果

- 在根 `.gitignore` 增加 `/plan/`，仅忽略仓库根目录的计划文件夹。
- `git check-ignore -v --no-index plan/.gitignore-probe` 验证命中该规则。
- `git ls-files -- plan` 显示目录内已有 19 个跟踪文件；依照非目标，本次未取消跟踪。
- 未涉及 Contract、业务层或跨 Feature 改动。

## 后续项

- 无。
