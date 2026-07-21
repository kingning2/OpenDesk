---
id: CHG-20260721-033-email-agent-docs-realignment
title: email-agent 迁入文档对齐（架构与 Domain 修订）
type: change
status: in_progress
priority: P0
owner: cursor-agent
domain: product
parent: EPIC-20260721-001-email-agent-port
depends_on: []
blocks: []
milestone: email-agent-port-phase0
created: 2026-07-21
updated: 2026-07-21
contracts: none
related:
  - EPIC-20260721-001-email-agent-port
  - ADR-0006-whatsapp-baileys-worker
  - ROADMAP-mvp-sales-workbench
---

# email-agent 迁入文档对齐（架构与 Domain 修订）

## 目标

将 email-agent 全量重写计划纳入 OpenDesk managed 文档体系：修订 WhatsApp 通道决策（Baileys Worker）、扩展 Mail/Customer/Pricing Domain、新建 Workflow/Analytics/Schedule/Alert/KOL 领域文档，并编写各实施阶段的分支命令手册。

## 非目标

- 本 Change **不写业务代码**、不改 Contract JSON、不跑 migration
- 不实现 KOL/Gbyte 对接（仅 kol domain planned 占位）
- 不迁移 email-agent 历史数据

## 背景

email-agent 业务能力将在 OpenDesk 技术栈内从零重写。现有 MVP 文档假定 WhatsApp Business API + Webhook，与已确认方案（Rust Worker Baileys、发送须人审、KOL 暂缓）冲突，须在写代码前完成文档对齐。

## 影响与边界

### 修改范围

- `docs/managed/roadmaps/` — MVP 路线图、分支命令手册
- `docs/managed/changes/2026/07/` — 新 Epic、本 Change
- `docs/managed/decisions/channel/` — ADR-0006；ADR-0004 标 superseded
- `docs/managed/domains/**` — channel/customer/mail/pricing 扩展；新建 workflow/analytics/schedule/alert/kol
- `docs/managed/MVP_REVIEW.md`、`epic-20260720-001` — 索引与 M5 表述
- `docs/managed/changes/.../chg-20260720-020` — 范围改为 Baileys Worker
- `docs/architecture/product-architecture.md`、`database-schema.md`
- `docs/architecture/whatsapp-webhook-deployment.md` — 废弃说明

### 不修改范围

- `contracts/`、`crates/`、`apps/`、`python/` 代码
- `domains/crawler/README.md`（获客与 KOL 平台对接无关）

## 验收

- [x] Epic + ADR-0006 已创建
- [x] Channel/Mail/Customer/Pricing Domain 已扩展
- [x] 新 Domain README（workflow/analytics/schedule/alert/kol）已创建
- [x] MVP 路线图、MVP_REVIEW、CHG-020 已对齐 Baileys
- [x] 分支命令手册已写入 `roadmaps/email-agent-port-branches.md`
- [x] `contracts/schema/v1/kol/_reserved` 占位已创建
- [ ] 团队评审通过后状态改为 `completed`

## 实际结果

- 新建：EPIC-20260721-001、ADR-0006、CHG-033、5 个 Domain README、email-agent-port-branches.md、kol contract 占位
- 修订：channel/customer/mail/pricing/domains README、mvp 路线图、MVP_REVIEW、epic-001、CHG-020、ADR-0004（superseded）、ADR-0001、product-architecture、database-schema、whatsapp-webhook-deployment（废弃说明）
- 分支命令：见 [email-agent-port-branches.md](../../roadmaps/email-agent-port-branches.md)

## 后续项

- [EPIC-20260721-001](epic-20260721-001-email-agent-port.md) Phase 0 起按分支手册实施代码
