# Channel Domain

## 职责

**WhatsApp Business API** 桌面接入，模式为 **辅助（Assist）**，不是自动客服。

- 收消息（webhook → Rust → 桌面 UI）
- 发消息（**人工点击发送** → Rust → Business API）
- 会话绑定 `customer_id`（邮箱/手机号关联）
- 为 AI 提供入站消息上下文（翻译、回复建议）
- 消息写入客户时间线

## 非职责

- 自动回复、无人值守发送
- AI 直接调用 WhatsApp 发送 API
- Webhook 公网 ingress 的完整 HA 运维（MVP 交付 [ADR-0004](../../decisions/channel/adr-0004-whatsapp-webhook-deployment.md) + [部署手册](../../../architecture/whatsapp-webhook-deployment.md)）
- 邮件（Mail 领域）
- Meta 账号注册与商务验证流程本身

## 稳定边界

```text
Meta WhatsApp Cloud API
  → Webhook Ingress（企业网关或 relay，Meta 可访问 HTTPS）
  → Rust channel adapter（验签、标准化、幂等）
  → SQLite + customer_timeline
  → Tauri Event → React 会话 UI

React 发送
  → IPC channel/send
  → Rust 校验 customer_id + 人工操作标记
  → Business API 出站
```

**与产品架构文档一致：** 见 [`docs/architecture/product-architecture.md`](../../../architecture/product-architecture.md)。

## 入口

| 类型 | 路径 |
|------|------|
| Rust | `crates/channel/`（骨架） |
| Contract | `contracts/schema/v1/channel/`（待建） |
| React | `apps/desktop/src/features/channel/`（骨架） |
| Epic 子任务 | [CHG-020](../../changes/2026/07/chg-20260720-020-whatsapp-business-assist.md) |
| ADR | [ADR-0004](../../decisions/channel/adr-0004-whatsapp-webhook-deployment.md) |
| 部署手册 | [`whatsapp-webhook-deployment.md`](../../../architecture/whatsapp-webhook-deployment.md) |

## 数据模型（MVP 目标）

### `channel_conversation`

| 字段 | 说明 |
|------|------|
| `id` | 主键 |
| `customer_id` | 外键 |
| `wa_phone` | WhatsApp 号码 |
| `last_message_at` | 最近消息时间 |

### `channel_message`

| 字段 | 说明 |
|------|------|
| `id` | 主键 |
| `conversation_id` | 外键 |
| `direction` | `inbound` / `outbound` |
| `body` | 原文 |
| `translated_body` | 翻译缓存（可选） |
| `wa_message_id` | 幂等键 |
| `sent_by` | `human` / — （MVP 出站均为 human） |
| `created_at` | 时间 |

## 当前状态

**尚未实现。** `crates/channel` 与 `features/channel` 为骨架；无 WhatsApp Contract。

## 当前约束

- **禁止** Agent 工具 `channel.send` 或等价自动发送
- 出站消息必须 `sent_by=human` 且经 UI 显式调用
- Webhook 必须先经 Rust，禁止 Meta → Python 直连
- 依赖 Customer M1/M4 与会话绑定策略
