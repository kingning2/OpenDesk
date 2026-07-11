---
id: CHG-20260711-001-agent-doc-gate
title: 将 Managed Docs 门禁接入根级 Agent 指令
status: completed
owner: codex
domain: documentation
created: 2026-07-11
updated: 2026-07-11
contracts: none
related: []
---

# 将 Managed Docs 门禁接入根级 Agent 指令

## 目标

让所有读取根级 `AGENTS.md` 的 AI Agent 在修改代码、契约、配置或依赖前，必须先登记 Change Record，并采用渐进式文档读取策略。

## 非目标

- 不修改业务代码或现有架构规则；
- 不接入 pre-commit 或 CI 自动检查；
- 不要求 Agent 默认读取全部 managed docs。

## 影响与边界

- 修改范围：根级 `AGENTS.md`、本 Change Record、活跃变更索引；
- 不修改范围：其他现有 Markdown、代码、契约和配置；
- Contract：无；
- 跨层：否；
- 跨 Feature：否；
- 风险：只依靠 Agent 遵守，尚无机械强制。

## 实施方案

1. 在根级 `AGENTS.md` 增加简短的 Managed Docs 强制入口。
2. 明确最小读取路径和“先记录、后实现”的状态门禁。
3. 链接详细治理规则，避免根指令持续膨胀。

## 验收

- [x] `AGENTS.md` 明确指向 Managed Docs 入口
- [x] 修改前必须创建 Change Record
- [x] Agent 不被要求递归读取全部文档
- [x] 完成后回填结果并清理 Active 索引
- [x] Markdown 链接检查通过

## 实际结果

已在根级 `AGENTS.md` 增加 Managed Docs 强制入口、渐进式读取顺序、Change Record 状态门禁、紧急修复规则和完成回填要求。所有新增相对链接均可解析，`git diff --check` 通过。未修改业务代码、Contract、配置或依赖。

## 后续项

- 可另行评估只校验、不自动生成记录的 pre-commit/CI 门禁。
