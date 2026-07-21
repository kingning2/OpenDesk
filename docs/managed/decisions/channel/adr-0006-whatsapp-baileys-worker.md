---
id: ADR-0006-whatsapp-baileys-worker
title: WhatsApp Baileys 协议桥在 opendesk-worker
type: adr
status: accepted
domain: channel
created: 2026-07-21
updated: 2026-07-21
deciders: product-owner
supersedes: ADR-0004-whatsapp-webhook-deployment
related:
  - CHG-20260720-020-whatsapp-business-assist
  - EPIC-20260721-001-email-agent-port
---

# WhatsApp Baileys 协议桥在 opendesk-worker

## Status

Accepted — 替代 [ADR-0004](adr-0004-whatsapp-webhook-deployment.md)（Business API + Webhook 方案）。

## Context

1. email-agent 生产环境使用 **Baileys 协议桥**（QR 登录、多账号、会话同步），迁入 OpenDesk 时需保留该能力。
2. 原 MVP 规划（ADR-0004）假定 **WhatsApp Business Cloud API + 公网 Webhook**，与迁入目标不一致。
3. OpenDesk 架构要求：重 IO/长连接任务在 **opendesk-worker**；Rust 主进程协调；Python 不直连渠道协议。
4. 产品约束不变：**禁止无人值守自动发送**；可保留发件队列，但出站须 UI 人工确认。

## Decision

### 1. 通道选型

| 项 | 决策 |
|----|------|
| 协议 | WhatsApp Web 协议（Baileys 或等价实现） |
| 运行位置 | **opendesk-worker** 子进程/任务（非 Tauri 主进程 UI 线程） |
| 登录 | QR 扫码；多账号状态由 Worker 维护 |
| 消息入库 | Worker → SQLite（`channel_*` 表）→ Tauri Event → React |
| 出站 | React `channel/send` IPC → Rust 校验 `customer_id` + 人工标记 → Worker 发信队列 |

### 2. 与 Rust 主进程分工

```text
React（会话 UI、QR 展示、人工发送）
  → Tauri IPC（channel/*）
  → Rust channel UseCase（校验、持久化、入队）
  → background_job / Worker IPC
  → opendesk-worker（Baileys bridge）
  → WhatsApp 网络

入站：Baileys 事件 → Worker 标准化 → 写 DB → Event → React
```

### 3. 禁止项（继承 MVP）

- Agent 工具 `channel.send` 或等价自动发送
- Python 直连 Baileys / WhatsApp 网络
- 无人值守自动回复 tick（可改为「建议草稿队列 + 人审」）

### 4. 技术实现选项（实施时选一，Phase 5 spike 定案）

| 选项 | 说明 |
|------|------|
| A | Worker 内嵌 Node 子进程运行 Baileys（薄封装，迁移风险低） |
| B | Rust 原生 WhatsApp 库（若成熟度足够） |
| C | Worker 侧独立 bridge 二进制 + JSON-RPC |

**不采用：** Meta Cloud API Webhook 作为主通道（ADR-0004 方案）。

## Alternatives

- **Business API + Webhook（ADR-0004）**：需 Meta 商务验证、公网 ingress；与 email-agent 现网能力不一致 → 不选
- **浏览器 CDP 镜像**：维护成本高、不稳定 → 不作为主通道，可后置评估

## Consequences

- **正面**：对齐 email-agent；桌面内 QR 登录；无需公网 webhook 隧道（S6b 验收改为协议登录 + 会话同步）
- **成本**：Baileys 协议变更风险；Worker 进程复杂度上升；需合规评估（非官方 API）
- **文档**：`domains/channel/README.md`、`CHG-020`、`whatsapp-webhook-deployment.md` 须同步修订
- **兼容**：`channel_conversation` / `channel_message` 表结构可复用；`wa_message_id` 改为协议层幂等键
