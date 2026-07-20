---
id: CHG-20260720-020-whatsapp-business-assist
title: WhatsApp Business 桌面辅助（翻译与建议，人发）
type: change
status: proposed
priority: P1
owner: developer
domain: channel
parent: EPIC-20260720-001-mvp-sales-workbench
depends_on:
  - CHG-20260720-013-customer-profile-model
  - CHG-20260720-017-ai-readonly-query-tools
  - CHG-20260720-019-customer-timeline-quote-history
  - CHG-20260720-027-llm-provider-settings
  - CHG-20260720-030-ai-correction-feedback
blocks: []
milestone: M5
created: 2026-07-20
updated: 2026-07-20
contracts: channel IPC + webhook DTO + agent suggest
related:
  - ADR-0001-ai-readonly-query-port
  - ADR-0004-whatsapp-webhook-deployment
  - ADR-0005-ai-correction-memory
---

# WhatsApp Business 桌面辅助（翻译与建议，人发）

## 目标

接入 WhatsApp Business API：**桌面端收消息、人工发送、AI 翻译入站消息、AI 建议回复**（注入客户档案 + 价目表）。**禁止自动回复。**

用户已确认：接 Business API 进桌面，通过桌面端发送消息。

## 非目标

- 自动发送、Bot 无人值守
- AI `channel.send` 工具
- 完整 webhook 高可用集群（MVP 文档说明部署选项即可）
- 邮件功能改动

## 背景

WhatsApp 用于获取信息、翻译、AI 建议怎么回——不是主谈价通道（邮件优先）。但仍需桌面收发以统一客户上下文。

## 影响与边界

### 修改范围

| 层级 | 路径 | 改动内容 |
|------|------|----------|
| Contract | `contracts/schema/v1/channel/dto/message.schema.json` | 消息 DTO |
| Contract | `contracts/schema/v1/channel/dto/conversation.schema.json` | 会话 DTO |
| Contract | `contracts/schema/v1/channel/ipc/send.request.schema.json` | 人工发送 |
| Contract | `contracts/schema/v1/channel/ipc/conversation_list.response.schema.json` | 会话列表 |
| Contract | `contracts/schema/v1/channel/webhook/inbound.schema.json` | 入站标准化事件 |
| Contract | `contracts/schema/v1/agent/ipc/wa_translate.request.schema.json` | 翻译 |
| Contract | `contracts/schema/v1/agent/ipc/wa_suggest.request.schema.json` | 回复建议 |
| Rust | `crates/channel/src/adapter/whatsapp/`（新建） | Cloud API 客户端 |
| Rust | `crates/channel/src/app/inbound_webhook.rs` | 验签、幂等、入库 |
| Rust | `crates/channel/src/app/send_message.rs` | **仅响应 UI IPC** |
| Rust | `crates/channel/src/app/bind_customer.rs` | WA 号绑 customer_id |
| Storage | `crates/storage/src/channel_db/` | conversation、message 表 |
| Rust | `crates/customer/src/app/timeline.rs` | wa_in / wa_out 条目 |
| Rust | `crates/app/` | channel IPC + webhook 路由 |
| Python | `python/packages/agent/agent/tasks/wa_translate.py` | 翻译 |
| Python | `python/packages/agent/agent/tasks/wa_suggest.py` | 建议（用 context_loader） |
| Frontend | `apps/desktop/src/features/channel/channel-page.tsx` | 会话 UI（替换占位） |
| Frontend | `apps/desktop/src/features/channel/use-channel-messages.ts` | 消息 hook |
| Frontend | `apps/desktop/src/features/channel/suggest-reply-panel.tsx` | 建议面板 |
| Frontend | `packages/platform/src/ipc/channel/` | IPC |
| Docs | `docs/managed/domains/channel/README.md` | 更新 |
| Docs | `docs/managed/changes/` 或 ADR | webhook 部署说明（ingress 选项） |
| Docs | `docs/architecture/whatsapp-webhook-deployment.md` | **MVP 必交付**操作手册 |
| Docs | `docs/managed/decisions/channel/adr-0004-whatsapp-webhook-deployment.md` | 架构决策 |

### 不修改范围

| 路径 | 原因 |
|------|------|
| `crates/mail/**` | 邮件主链路独立 |
| Agent 自动 send 工具 | **禁止** |
| `product-architecture.md` 自动客服描述 | 单独 Change |
| Python WhatsApp 客户端 | 渠道只在 Rust |

### Contract

- **需要** channel + agent wa_* schema

### 跨层

- Meta webhook → Rust → DB → Event → React
- React send → Rust → Meta
- React suggest → Rust → Python → Rust Query Port（只读）→ Python

### 跨 Feature

- Channel UI 跳转 customer 详情；绑 customer_id

### 风险

- Webhook 需公网 HTTPS：按 [ADR-0004](../../decisions/channel/adr-0004-whatsapp-webhook-deployment.md) 与 [`whatsapp-webhook-deployment.md`](../../../architecture/whatsapp-webhook-deployment.md) 完成 dev 隧道验收
- 出站必须校验「人工发送」标志，防误接自动流程

## 依赖关系

- 父任务：EPIC-20260720-001
- 前置任务：CHG-013、CHG-017、CHG-019
- 阻塞：无

## 实施方案

1. Contract + channel_db + Cloud API 发送/收信基础。
2. Webhook ingress：最小 relay 或 dev tunnel；Rust 验签 Meta signature。
3. 会话列表 + 消息流 UI；绑 customer（phone 匹配或手动选客户）。
4. 入站消息：自动调 wa_translate 显示译文（可配置）。
5. 「建议回复」：wa_suggest + context_loader；用户点选复制/填入输入框。
6. 「发送」按钮：仅 channel/send IPC；**无**「建议并发送」。
7. timeline 写入 wa_in / wa_out。

## 验收

- [ ] Business API 收发消息成功（测试号）
- [ ] 消息关联 customer_id
- [ ] 翻译与建议可用且含客户/价目上下文
- [ ] 无自动发送代码路径
- [ ] Agent 工具列表无 channel.send
- [ ] **开发隧道联调**：测试号入站消息在 UI 可见（见 ADR-0004）
- [ ] 架构检查通过

## 实际结果

（完成前留空。）

## 后续项

- IMAP 收信 Change（MVP 后）
- 修正 product-architecture.md 叙事
