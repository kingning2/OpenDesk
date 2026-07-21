# OpenDesk MVP — 团队评审指南

本文是 **提交 GitHub 供团队查阅** 的入口。只读规划文档，不含可运行 MVP 产品代码。

## 产品是什么（30 秒）

**商务工作台**，不是 WhatsApp 自动客服：

```text
YouTube 爬邮箱 → 客户档案 → 自有 SMTP/IMAP 邮件谈价 → WhatsApp Baileys 辅助（翻译/建议，人发）
```

AI **记得每个客户**（来源、报价、合作状态），但 **只能只读查库**，不能改数据、不能自动发信。

**email-agent 迁入：** 工作流、收件箱、待发人审、统计等 — 见 [EPIC-20260721-001](changes/2026/07/epic-20260721-001-email-agent-port.md)。**KOL/Gbyte 对接暂缓。**

## 从哪里开始读

按顺序阅读即可，无需递归整个 `docs/managed/`：

| 顺序 | 文档 | 你会了解到 |
|------|------|------------|
| 1 | [MVP 总路线图](roadmaps/mvp-sales-workbench.md) | 成功标准、M1–M6 里程碑、文档地图 |
| 1b | [email-agent 分支手册](roadmaps/email-agent-port-branches.md) | 各 Phase 的 `pnpm branch:create` 命令 |
| 2 | [产品架构](../architecture/product-architecture.md) | 三端职责、端到端流程图、客户字段 B |
| 2b | [数据库设计](../architecture/database-schema.md) | **全表 DDL**、双库、OCR / background_job |
| 2c | [进程模型](../architecture/process-model.md) | **Worker 独立进程**；OCR 不得阻塞 UI |
| 3 | [Epic MVP](changes/2026/07/epic-20260720-001-mvp-sales-workbench.md) | 子任务依赖与总体验收 |
| 3b | [Epic email-agent](changes/2026/07/epic-20260721-001-email-agent-port.md) | 迁入范围与 Phase |
| 4 | [ADR-0001](decisions/customer/adr-0001-ai-readonly-query-port.md) | AI 只读查库 |
| 5 | [ADR-0002](decisions/runtime/adr-0002-heavy-work-worker-process.md) | 重任务 Worker 进程 |
| 5b | [ADR-0003](decisions/ocr/adr-0003-tesseract-local-model-on-demand-download.md) | Tesseract + 语言包按需下载 |
| 5c | [ADR-0006](decisions/channel/adr-0006-whatsapp-baileys-worker.md) | **WhatsApp Baileys Worker**（当前方案） |
| 5d | [ADR-0005](decisions/agent/adr-0005-ai-correction-memory.md) | AI 纠错记忆 |
| 6 | Child Change `013`–`033` | 每次改动的精确范围与验收 |

## 领域文档索引

| 领域 | 文档 | 状态 |
|------|------|------|
| Customer | [domains/customer/README.md](domains/customer/README.md) | planned |
| Mail | [domains/mail/README.md](domains/mail/README.md) | planned |
| Workflow | [domains/workflow/README.md](domains/workflow/README.md) | planned |
| Crawler | [domains/crawler/README.md](domains/crawler/README.md) | 部分已实现 |
| Pricing | [domains/pricing/README.md](domains/pricing/README.md) | planned |
| Channel (WA) | [domains/channel/README.md](domains/channel/README.md) | planned（Baileys） |
| Analytics | [domains/analytics/README.md](domains/analytics/README.md) | planned |
| Schedule | [domains/schedule/README.md](domains/schedule/README.md) | planned |
| Alert | [domains/alert/README.md](domains/alert/README.md) | planned |
| KOL | [domains/kol/README.md](domains/kol/README.md) | **暂缓（仅占位）** |
| Runtime/Worker | [domains/runtime/README.md](domains/runtime/README.md) | planned |
| Storage | [domains/storage/README.md](domains/storage/README.md) | planned |
| OCR | [domains/ocr/README.md](domains/ocr/README.md) | planned |
| Agent | [domains/agent/README.md](domains/agent/README.md) | 骨架 + Ping |

## 子任务一览（实施顺序）

| ID | 里程碑 | 标题 | Change Record |
|----|--------|------|---------------|
| CHG-013 | M1 | 客户档案模型与详情页 | [chg-20260720-013](changes/2026/07/chg-20260720-013-customer-profile-model.md) |
| CHG-014 | M1 | 爬虫结果导入客户 | [chg-20260720-014](changes/2026/07/chg-20260720-014-crawler-lead-import.md) |
| CHG-015 | M2 | SMTP + **邮件模板** + 发信 | [chg-20260720-015](changes/2026/07/chg-20260720-015-smtp-mail-send.md) |
| CHG-016 | M3 | 价目表与阶梯报价 | [chg-20260720-016](changes/2026/07/chg-20260720-016-pricing-catalog.md) |
| CHG-017 | M3 | AI 只读 Query Port | [chg-20260720-017](changes/2026/07/chg-20260720-017-ai-readonly-query-tools.md) |
| CHG-018 | M3 | AI 邮件润色（基于模板） | [chg-20260720-018](changes/2026/07/chg-20260720-018-ai-mail-draft.md) |
| CHG-019 | M4 | 报价/合作审计 | [chg-20260720-019](changes/2026/07/chg-20260720-019-customer-timeline-quote-history.md) |
| CHG-020 | M5 | WhatsApp **Baileys** 辅助 | [chg-20260720-020](changes/2026/07/chg-20260720-020-whatsapp-business-assist.md) |
| CHG-022 | M0 | 数据库 + Worker + OCR **设计**（已完成） | [chg-20260720-022](changes/2026/07/chg-20260720-022-database-worker-ocr-design.md) |
| CHG-023 | M6 | opendesk.db 队列 + Worker 二进制骨架 | [chg-20260720-023](changes/2026/07/chg-20260720-023-opendesk-db-worker-skeleton.md) |
| CHG-024 | M6 | Tesseract OCR 管线（仅 Worker） | [chg-20260720-024](changes/2026/07/chg-20260720-024-ocr-worker-pipeline.md) |
| CHG-025 | M6 | **语言包按需下载** | [chg-20260720-025](changes/2026/07/chg-20260720-025-ocr-tesseract-model-download.md) |
| CHG-026 | M2 | **客户邮件回复记录** | [chg-20260720-026](changes/2026/07/chg-20260720-026-mail-inbound-reply-record.md) |
| CHG-027 | M3 | **LLM Provider 配置 UI** | [chg-20260720-027](changes/2026/07/chg-20260720-027-llm-provider-settings.md) |
| CHG-028 | M6 | **OCR 商务场景** | [chg-20260720-028](changes/2026/07/chg-20260720-028-ocr-business-scenarios.md) |
| CHG-029 | M2 | **IMAP 自动收信** | [chg-20260720-029](changes/2026/07/chg-20260720-029-imap-inbound-sync.md) |
| CHG-030 | M3 | **AI 纠错记忆** | [chg-20260720-030](changes/2026/07/chg-20260720-030-ai-correction-feedback.md) |
| CHG-031 | M1 | **Playwright 邮箱补全** | [chg-20260720-031](changes/2026/07/chg-20260720-031-crawler-playwright-email-enrich.md) |
| CHG-033 | — | **email-agent 文档对齐** | [chg-20260721-033](changes/2026/07/chg-20260721-033-email-agent-docs-realignment.md) |

每份 Change 均包含：**修改范围**、**不修改范围**、Contract 影响、验收清单。

## 当前代码仓库现状

| 已有 | 尚无（规划内） |
|------|----------------|
| UI Shell、YouTube 爬虫、crawler.db | opendesk.db、客户 CRM |
| Agent Ping、Sidecar | 邮件模板、SMTP、Worker、OCR |
| email-agent 迁入 **文档**（CHG-033） | email-agent 业务能力代码 |

## 已废弃的叙事

- 「WhatsApp AI 自动客服 Desktop Agent」
- 「WhatsApp **Business API + Webhook** 为主通道」（已由 ADR-0006 Baileys 替代 ADR-0004）
- 「阶段 1：WhatsApp MVP = 自动客服闭环」

叙事修正：[CHG-021](changes/2026/07/chg-20260720-021-product-narrative-realignment.md)、[CHG-033](changes/2026/07/chg-20260721-033-email-agent-docs-realignment.md)

## 评审时可关注的点

1. M1–M5 顺序与依赖是否合理？
2. email-agent 迁入与 MVP 扩展边界是否清晰？KOL 暂缓是否可接受？
3. **Baileys Worker**（ADR-0006）vs 原 Business API 的风险是否可接受？
4. `outreach_stage` 与 `lifecycle_status` 双字段是否够用？
5. 待发队列「人审确认」是否符合操作习惯？
6. 分支手册 [email-agent-port-branches.md](roadmaps/email-agent-port-branches.md) 是否覆盖你的协作方式？
7. 其余项见原 MVP 评审清单（客户字段、AI 工具、OCR、LLM 配置等）。

## 文档治理

- 开始任何 **代码** 工作前：对应 Change 状态 `approved` → `in_progress`，登记 `registry/ACTIVE.md`
- 规则详见 [GOVERNANCE.md](GOVERNANCE.md)
