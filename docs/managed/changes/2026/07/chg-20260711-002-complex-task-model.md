---
id: CHG-20260711-002-complex-task-model
title: 完善复杂任务父子关系与路线图管理
type: change
status: completed
priority: P1
owner: codex
domain: documentation
parent: none
depends_on: []
blocks: []
milestone: docs-governance-v1
created: 2026-07-11
updated: 2026-07-11
contracts: none
related:
  - CHG-20260711-001-agent-doc-gate
---

# 完善复杂任务父子关系与路线图管理

## 目标

支持复杂任务拆成稳定关联的 Epic、Child Change 和 Milestone Roadmap，同时保持文档路径稳定、索引短小和按需读取。

## 非目标

- 不采用按状态移动文件的目录模型；
- 不建立包含全部历史的全局执行队列；
- 不修改业务代码、Contract 或依赖。

## 影响与边界

- 修改范围：Managed Docs 治理规则、上下文策略、Active 索引、模板与文档领域入口；
- 不修改范围：现有业务文档与代码；
- Contract：无；
- 跨层：否；
- 跨 Feature：否；
- 风险：元数据字段增加，需要保持模板简洁。

## 实施方案

1. 为 Change 增加类型、优先级、父任务、依赖、阻塞和里程碑字段。
2. 新增 Epic 和 Roadmap 模板。
3. 明确复杂任务拆分、父子关系和读取预算。
4. 扩充 Active 索引的最小关系字段。

## 验收

- [x] Change 模板支持父子任务与依赖
- [x] Epic 模板只承载总目标、子任务和总验收
- [x] Roadmap 模板按领域和里程碑管理
- [x] Active 索引支持并行任务且保持短小
- [x] 明确禁止状态目录移动和无限总队列
- [x] 所有 Markdown 相对链接有效

## 实际结果

已增加 `type`、`priority`、`parent`、`depends_on`、`blocks` 和 `milestone` 元数据，新增 Epic、Roadmap 模板及 Documentation Domain，完善复杂任务拆分和上下文读取规则。所有文档保留固定路径，状态仅通过 Front Matter 和 Active 索引更新。相对链接检查与 `git diff --check` 均通过，没有文件达到 200 行或 15 KiB。

## 后续项

- 可另行增加元数据静态校验脚本。
