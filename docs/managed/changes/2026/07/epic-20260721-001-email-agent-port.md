---
id: EPIC-20260721-001-email-agent-port
title: email-agent 业务能力迁入 OpenDesk
type: epic
status: proposed
priority: P0
owner: product-owner
domain: product
milestone: email-agent-port
created: 2026-07-21
updated: 2026-07-21
related:
  - EPIC-20260720-001-mvp-sales-workbench
  - ROADMAP-mvp-sales-workbench
  - ADR-0006-whatsapp-baileys-worker
  - CHG-20260721-033-email-agent-docs-realignment
---

# email-agent 业务能力迁入 OpenDesk

## 目标

在 OpenDesk 架构（React → Rust → Python，Contract First）内 **从零重写** email-agent 业务能力（非文件迁移）：邮件、工作流、话术、预警、排期、统计、WhatsApp（Baileys Worker）、定价引擎等；**扩展** 而非替代 [EPIC-20260720-001](epic-20260720-001-mvp-sales-workbench.md)。

## 非目标

- 搬运 email-agent Express/FastAPI/Node 运行时
- KOL/Gbyte 平台对接（见 [domains/kol/README.md](../../domains/kol/README.md)，Phase 7 独立 Epic）
- 无人值守自动发送（保留队列/批量，**须人工确认发送**）

## 总体边界

| 项 | 说明 |
|----|------|
| 主领域 | product（跨 customer / mail / workflow / channel / pricing / analytics） |
| 与 MVP 关系 | CHG-013~031 为地基；本 Epic 在各 Change 上 **扩展** email-agent 能力 |
| 长期决策 | [ADR-0006](../../decisions/channel/adr-0006-whatsapp-baileys-worker.md)（替代 ADR-0004） |
| 实施顺序 | Contract → Codegen → Rust → Python → React |
| 分支规范 | 见 [email-agent-port-branches.md](../../roadmaps/email-agent-port-branches.md) |

## 子任务（规划）

| Phase | Child Change | 状态 | 说明 |
|-------|--------------|------|------|
| 0 | [CHG-033 文档对齐](chg-20260721-033-email-agent-docs-realignment.md) | in_progress | 本批文档 |
| 0 | CHG-034 phase0-contracts（待建） | proposed | Contract + KOL 预留 |
| 1 | 扩展 CHG-013 | proposed | outreach_stage + workflow |
| 2 | 扩展 CHG-015/026/029 | proposed | 收件箱 + 多账号同步 |
| 3 | 扩展 CHG-018/030 | proposed | 待发队列人审 |
| 4 | 扩展 CHG-016 | proposed | YT 定价引擎 |
| 5 | 扩展 CHG-020 | proposed | Baileys Worker（ADR-0006） |
| 6 | 待建 | proposed | 统计/排期/数据迁移 |
| 7 | 待建 | proposed | KOL 系统迁入（独立） |

## 总体验收

- [ ] CHG-033 completed；文档与 ADR 评审通过
- [ ] Phase 0~6 对应 Change 全部 completed
- [ ] email-agent 核心 Tab（除 KOL）在 OpenDesk 桌面可演示
- [ ] 发送路径均须人工确认；无 Agent 自动发信
- [ ] `pnpm lint` 与架构检查通过

## 结果摘要

（Epic 完成时填写。）
