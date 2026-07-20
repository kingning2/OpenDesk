---
id: CHG-20260715-003-ignore-local-config
title: 忽略本地生成配置与环境文件
status: completed
owner: codex
domain: documentation
created: 2026-07-15
updated: 2026-07-15
---

# 快速修复

## 问题与目标

自动生成的当前分支规则和本地环境文件可能被误加入版本控制；需要忽略它们，同时创建并保留可提交的环境变量示例。

## 边界

- 修改：`.gitignore`、`.env.example`、本地 `.env` 占位文件、本 Change Record 与 Active Registry。
- 不修改：仓库已跟踪的 `.cursor` 规则；不写入任何真实密钥。
- Contract/跨层影响：无。

## 验收

- [x] 原问题可复现或证据明确
- [x] 修复验证通过
- [x] 无架构边界变化

## 结果

- `.gitignore` 忽略 `.cursor/rules/active-branch.mdc`、`.env` 和 `.env.*`，并对白名单 `*.example` 取反。
- 新增 `.env.example` 与被忽略的本地 `.env`，仅包含空密钥和本地开发模型配置。
- 将先前暂存的生成文件从 Git 索引移除，工作区文件保留。
- 验证：`git check-ignore -v --no-index` 正确命中生成规则和 `.env`，`.env.example` 保持可跟踪。
