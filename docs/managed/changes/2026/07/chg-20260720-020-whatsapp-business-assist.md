---
id: CHG-20260720-020-whatsapp-business-assist
title: WhatsApp 桌面辅助（Baileys Worker，翻译与建议，人发）
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
  - CHG-20260720-023-opendesk-db-worker-skeleton
blocks: []
milestone: M5
created: 2026-07-20
updated: 2026-07-21
contracts: channel IPC + protocol events + agent suggest
related:
  - ADR-0001-ai-readonly-query-port
  - ADR-0006-whatsapp-baileys-worker
  - ADR-0005-ai-correction-memory
  - EPIC-20260721-001-email-agent-port
---

# WhatsApp 桌面辅助（Baileys Worker，翻译与建议，人发）

## 目标

在 **opendesk-worker** 内接入 WhatsApp **Baileys 协议桥**（[ADR-0006](../../decisions/channel/adr-0006-whatsapp-baileys-worker.md)）：QR 登录、多账号、桌面收消息、**人工发送**、AI 翻译入站消息、AI 建议回复（注入客户档案 + 价目表）。**禁止自动回复。**

## 非目标

- 自动发送、Bot 无人值守、自动回复 tick
- AI `channel.send` 工具
- Meta WhatsApp Business Cloud API / 公网 Webhook（见已 superseded 的 ADR-0004）
- 邮件功能改动
- CDP 浏览器镜像（后置）

## 背景

WhatsApp 用于获取信息、翻译、AI 建议怎么回——不是主谈价通道（邮件优先）。email-agent 迁入要求保留 Baileys 协议能力，在 OpenDesk Worker 架构内重写。

## 影响与边界

### 修改范围

| 层级 | 路径 | 改动内容 |
|------|------|----------|
| Contract | `contracts/schema/v1/channel/dto/message.schema.json` | 消息 DTO |
| Contract | `contracts/schema/v1/channel/dto/conversation.schema.json` | 会话 DTO |
| Contract | `contracts/schema/v1/channel/dto/account.schema.json` | 账号与协议状态 |
| Contract | `contracts/schema/v1/channel/ipc/send.request.schema.json` | 人工发送 |
| Contract | `contracts/schema/v1/channel/ipc/conversation_list.response.schema.json` | 会话列表 |
| Contract | `contracts/schema/v1/channel/event/protocol_*.schema.json` | QR/连接/入站事件 |
| Contract | `contracts/schema/v1/agent/ipc/wa_translate.request.schema.json` | 翻译 |
| Contract | `contracts/schema/v1/agent/ipc/wa_suggest.request.schema.json` | 回复建议 |
| Rust | `crates/channel/src/adapter/protocol/`（新建） | Worker 桥接客户端 |
| Rust | `crates/channel/src/app/sync_inbound.rs` | 入站标准化、幂等、入库 |
| Rust | `crates/channel/src/app/send_message.rs` | **仅响应 UI IPC** → Worker 队列 |
| Rust | `crates/channel/src/app/bind_customer.rs` | WA 号绑 customer_id |
| Worker | `crates/worker/src/handlers/wa_protocol_bridge.rs` | Baileys 子进程/桥 |
| Storage | `crates/storage/src/channel_db/` | account、conversation、message |
| Rust | `crates/customer/src/app/timeline.rs` | wa_in / wa_out |
| Rust | `crates/app/` | channel IPC |
| Python | `python/packages/agent/agent/tasks/wa_translate.py` | 翻译 |
| Python | `python/packages/agent/agent/tasks/wa_suggest.py` | 建议 |
| Frontend | `apps/desktop/src/features/channel/` | 会话 UI、QR 登录 |
| Docs | `docs/managed/domains/channel/README.md` | 已更新 ADR-0006 |

### 不修改范围

| 路径 | 原因 |
|------|------|
| `crates/mail/**` | 邮件主链路独立 |
| Agent 自动 send 工具 | **禁止** |
| `docs/architecture/whatsapp-webhook-deployment.md` | 已废弃（Business API） |

### 跨层

- Worker Baileys → Rust → DB → Event → React
- React send → Rust → Worker（人工标记）
- React suggest → Rust → Python → Query Port → Python

### 风险

- Baileys 协议稳定性；Phase 5 须先完成技术 spike
- 出站必须校验「人工发送」标志

## 依赖关系

- 父任务：EPIC-20260720-001、[EPIC-20260721-001](epic-20260721-001-email-agent-port.md)
- 前置：CHG-013、CHG-017、CHG-019、**CHG-023**（Worker 骨架）
- 架构决策：[ADR-0006](../../decisions/channel/adr-0006-whatsapp-baileys-worker.md)

## 实施方案

1. Phase 5 spike：Worker 内 Baileys 可行方案（A/B/C，见 ADR-0006）。
2. Contract + channel_db + 协议事件。
3. QR 登录 UI；多账号状态。
4. 会话列表 + 消息流；绑 customer。
5. 入站翻译、建议回复；**发送仅 IPC**。
6. timeline 写入 wa_in / wa_out。

## 验收

- [ ] Baileys QR 登录成功，至少 1 个账号 `connected`
- [ ] 收消息入库并在 UI 显示
- [ ] 人工发送成功；无自动发送路径
- [ ] 翻译与建议含客户/价目上下文
- [ ] Agent 工具列表无 `channel.send`
- [ ] 架构检查通过

## 实际结果

（完成前留空。）

## 后续项

- CDP 镜像（可选，低优先）
