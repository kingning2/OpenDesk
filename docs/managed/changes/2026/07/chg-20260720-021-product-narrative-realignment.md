---
id: CHG-20260720-021-product-narrative-realignment
title: 产品叙事修正为商务工作台并整理 GitHub 评审文档
type: change
status: completed
priority: P0
owner: product-owner
domain: documentation
parent: EPIC-20260720-001-mvp-sales-workbench
depends_on: []
blocks: []
milestone: M0-docs
created: 2026-07-20
updated: 2026-07-20
contracts: none
related:
  - ROADMAP-mvp-sales-workbench
  - CHG-20260715-001-whatsapp-architecture-docs
---

# 产品叙事修正为商务工作台并整理 GitHub 评审文档

## 目标

1. **删除/替换** 错误的「WhatsApp 自动客服」产品叙事；
2. 对齐真实业务：YouTube 获客 → SMTP 邮件谈价 → WA 辅助（译/建议/人发）；
3. 整理 MVP 规划文档包，**可提交 GitHub** 供团队评审；
4. **不开始** MVP 业务代码实施。

## 非目标

- 不实现 CHG-013 ~ CHG-020 任何代码
- 不删除 YouTube 爬虫等已有垂直切片代码
- 不修改 Contract、Rust/Python 业务逻辑

## 背景

2026-07-15 的 [CHG-20260715-001](chg-20260715-001-whatsapp-architecture-docs.md) 引入了 WhatsApp AI 自动客服叙事，与产品负责人确认的真实业务不符。需在不动代码前提下修正文档并提交评审。

## 影响与边界

### 修改范围

| 文件 | 操作 | 说明 |
|------|------|------|
| `docs/architecture/product-architecture.md` | **重写** | 商务工作台产品架构 |
| `docs/architecture/python-ai-runtime-architecture.md` | **部分更新** | 定位、职责、阶段 1、相关文档链接 |
| `docs/architecture/README.md` | 更新 | 评审入口与产品描述 |
| `docs/PROJECT_ANALYSIS.md` | **重写** | 指向 MVP_REVIEW |
| `README.md` | 更新 | 产品描述 + MVP 评审链接 |
| `docs/managed/MVP_REVIEW.md` | **新建** | GitHub 团队评审入口 |
| `docs/managed/README.md` | 更新 | MVP 规划说明 |
| `docs/managed/roadmaps/mvp-sales-workbench.md` | 更新 | 叙事修正引用 |
| `docs/managed/domains/**` | 新建/更新 | customer/mail/crawler/pricing/channel/agent |
| `docs/managed/decisions/customer/adr-0001-*.md` | 新建 | AI 只读查库 |
| `docs/managed/changes/2026/07/epic-20260720-001-*.md` | 新建 | Epic |
| `docs/managed/changes/2026/07/chg-20260720-013~020-*.md` | 新建 | 8 个子任务 Change |
| `docs/managed/changes/.gitignore` | 更新 | 白名单 MVP 文档可提交 |
| `docs/managed/registry/DOMAINS.md` | 更新 | 新领域注册 |
| `docs/managed/roadmaps/README.md` | 更新 | 路线图索引 |

### 不修改范围

| 路径 | 原因 |
|------|------|
| `apps/desktop/**` | 无代码实施 |
| `crates/**`（除文档引用路径说明） | 无代码实施 |
| `python/**` | 无代码实施 |
| `contracts/**` | 无契约变更 |
| `skills/**` | 不在本次范围 |
| 历史 Change `chg-20260715-001-*.md` | 保留作历史证据，不篡改 completed 正文 |

### 已废弃叙事（勿再引用）

- 「WhatsApp AI Desktop Agent」自动客服
- 「阶段 1：WhatsApp MVP = 自动回复闭环」
- 「统一 AI 智能客服平台」作为产品主描述

## 依赖关系

- 父任务：EPIC-20260720-001-mvp-sales-workbench
- 前置任务：无
- 阻塞：CHG-013 等代码 Change 应在团队评审后再 `approved`

## 实施方案

1. 重写 `product-architecture.md` 为商务工作台架构。
2. 创建 `MVP_REVIEW.md` 作为 GitHub 唯一评审入口。
3. 建立 managed 文档包（路线图、Epic、Domain、ADR、Child Change）。
4. 更新 `changes/.gitignore` 白名单以便 git 跟踪 MVP 规划文件。
5. 更新根 README 与 PROJECT_ANALYSIS 指向新入口。

## 验收

- [x] 无「WhatsApp 自动客服」作为主产品描述
- [x] MVP_REVIEW 可链到全部 8 个 Child Change
- [x] 每个 Child Change 含修改范围 / 不修改范围
- [x] ADR-0001 明确 AI 只读、禁止写库
- [x] gitignore 放行 MVP 规划文件
- [x] 未修改业务代码与 Contract

## 实际结果

- 产品架构文档已重写为商务工作台叙事。
- 新增 `docs/managed/MVP_REVIEW.md` 及完整 MVP 规划文档包。
- `changes/.gitignore` 已白名单 Epic + CHG-013~021。
- 根 README、PROJECT_ANALYSIS、architecture README 已更新。
- **未**启动 CHG-013 代码工作。

## 后续项

- 团队评审通过后，将 CHG-013 状态改为 `approved` 并开始 M1 实施。
- 可选：在 GitHub 开 Discussion/PR 专门评审 `docs/managed/MVP_REVIEW.md`。
