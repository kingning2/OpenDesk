---
id: EPIC-20260720-001-mvp-sales-workbench
title: MVP 商务工作台
type: epic
status: proposed
priority: P0
owner: product-owner
domain: product
milestone: mvp-sales-workbench
created: 2026-07-20
updated: 2026-07-21
related:
  - ROADMAP-mvp-sales-workbench
  - ADR-0001-ai-readonly-query-port
  - EPIC-20260721-001-email-agent-port
---

# MVP 商务工作台

## 目标

交付可演示的商务闭环：YouTube 邮箱获客 → 客户档案 → 自有 SMTP/IMAP 邮件谈价 → AI 只读查库并起草邮件（含纠错记忆）→ WhatsApp **Baileys 协议桥**桌面辅助（翻译/建议，人发）。

**扩展：** [EPIC-20260721-001](epic-20260721-001-email-agent-port.md) 在 MVP 地基上迁入 email-agent 工作流、收件箱、待发人审、统计等能力。

## 非目标

- 全自动 WhatsApp/邮件回复
- AI 修改客户数据、报价、合作状态
- 多渠道获客、ERP 工具、多租户
- 修正 `product-architecture.md` 全文（已由 CHG-021 完成）

## 总体边界

| 项 | 说明 |
|----|------|
| 主领域 | product（跨 customer / mail / crawler / pricing / channel / agent） |
| Contract | 每个 Child Change 列出新增/修改 schema；Epic 不重复 schema 正文 |
| 跨层 | 全部遵循 React → Rust → Python |
| 长期决策 | [ADR-0001](../../decisions/customer/adr-0001-ai-readonly-query-port.md)、[ADR-0005](../../decisions/agent/adr-0005-ai-correction-memory.md)、[ADR-0006](../../decisions/channel/adr-0006-whatsapp-baileys-worker.md) |
| 数据所有权 | 客户/报价/合作/邮件记录 → Rust + SQLite；Python 无持久化写权限 |

## 子任务

| Child Change | 状态 | 里程碑 | Depends on | 验收结果 |
|--------------|------|--------|------------|----------|
| [CHG-013 客户档案模型与详情页](chg-20260720-013-customer-profile-model.md) | proposed | M1 | — | — |
| [CHG-014 爬虫结果导入客户](chg-20260720-014-crawler-lead-import.md) | proposed | M1 | CHG-013 | — |
| [CHG-015 SMTP + 邮件模板](chg-20260720-015-smtp-mail-send.md) | proposed | M2 | CHG-013 | — |
| [CHG-016 价目表与阶梯报价](chg-20260720-016-pricing-catalog.md) | proposed | M3 | CHG-013 | — |
| [CHG-017 AI 只读 Query Port](chg-20260720-017-ai-readonly-query-tools.md) | proposed | M3 | CHG-013, CHG-016, ADR-0001 | — |
| [CHG-018 AI 邮件润色（基于模板）](chg-20260720-018-ai-mail-draft.md) | proposed | M3 | CHG-015, CHG-016, CHG-017 | — |
| [CHG-019 报价/合作变更与审计](chg-20260720-019-customer-timeline-quote-history.md) | proposed | M4 | CHG-013, CHG-015 | — |
| [CHG-020 WhatsApp 辅助（Baileys）](chg-20260720-020-whatsapp-business-assist.md) | proposed | M5 | CHG-013, CHG-019, CHG-023 | — |
| [CHG-022 数据库+Worker+OCR 设计](chg-20260720-022-database-worker-ocr-design.md) | completed | M0-infra | CHG-021 | — |
| [CHG-023 opendesk.db + Worker 骨架](chg-20260720-023-opendesk-db-worker-skeleton.md) | proposed | M6 | CHG-022 | — |
| [CHG-024 OCR Worker 管线](chg-20260720-024-ocr-worker-pipeline.md) | proposed | M6 | CHG-023, CHG-025, CHG-013 | — |
| [CHG-025 Tesseract 语言包下载](chg-20260720-025-ocr-tesseract-model-download.md) | proposed | M6 | CHG-023 | — |
| [CHG-026 客户邮件回复记录](chg-20260720-026-mail-inbound-reply-record.md) | proposed | M2 | CHG-013, CHG-015 | — |
| [CHG-027 LLM Provider 配置 UI](chg-20260720-027-llm-provider-settings.md) | proposed | M3 | — | — |
| [CHG-028 OCR 商务场景](chg-20260720-028-ocr-business-scenarios.md) | proposed | M6 | CHG-024, CHG-017, CHG-013 | — |
| [CHG-029 IMAP 自动收信](chg-20260720-029-imap-inbound-sync.md) | proposed | M2 | CHG-026, CHG-023 | — |
| [CHG-030 AI 纠错记忆](chg-20260720-030-ai-correction-feedback.md) | proposed | M3 | CHG-017, CHG-027, ADR-0005 | — |
| [CHG-031 Playwright 邮箱补全](chg-20260720-031-crawler-playwright-email-enrich.md) | proposed | M1 | CHG-023 | — |

## 总体验收

- [ ] M1：S1 通过（客户建档 + 爬虫导入 + **两阶段邮箱：API + Playwright 补全，不丢弃**）
- [ ] M2：S2 + S2b + S2c + **S2d** 通过（发信 + 入站 **IMAP** + 手动兜底）
- [ ] M3：S4 + S5 + S10 + **S12** 通过（AI + LLM + **纠错记忆**）
- [ ] M4：S3 通过（正式合作字段 + 报价史）
- [ ] M5：S6 + **S6b** 通过（WA **Baileys** 辅助 + **QR 登录与会话同步**）
- [ ] M6：S7 + S8 + S9 + **S11** 通过（Worker OCR + 语言包 + **商务场景**）
- [ ] 各 Domain README 已更新为最终现状
- [ ] `python skills/opendesk/scripts/check_architecture.py` 通过

## 结果摘要

（Epic 完成时填写。）
