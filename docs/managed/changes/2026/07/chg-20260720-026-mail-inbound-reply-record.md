---
id: CHG-20260720-026-mail-inbound-reply-record
title: 客户邮件回复记录（手动录入兜底）
type: change
status: proposed
priority: P1
owner: developer
domain: mail
parent: EPIC-20260720-001-mvp-sales-workbench
depends_on:
  - CHG-20260720-013-customer-profile-model
  - CHG-20260720-015-smtp-mail-send
blocks:
  - CHG-20260720-029-imap-inbound-sync
milestone: M2
created: 2026-07-20
updated: 2026-07-20
contracts: mail inbound IPC + DTO
related:
  - CHG-20260720-019-customer-timeline-quote-history
---

# 客户邮件回复记录（手动录入兜底）

## 目标

定义入站邮件数据模型，并提供 **手动录入兜底**（从 Outlook/Gmail 复制粘贴），写入 `mail_message`（`direction=inbound`）与 `customer_timeline`（`email_received`）。

**主路径为 IMAP 自动收信（CHG-029）**；本 Change 负责共用数据模型 + 下列场景的手动补录：

- IMAP 未匹配到客户的邮件，人工关联前临时记录
- 使用非 IMAP 邮箱、转发到个人邮箱后粘贴
- IMAP 同步失败期间的应急录入

## 非目标

- IMAP 自动同步（[CHG-029](chg-20260720-029-imap-inbound-sync.md)）
- AI 自动解析邮件并改客户字段
- 附件存储（MVP 可后补）

## 背景

入站邮件须进入时间线与 AI 上下文。自动收信由 CHG-029 实现；本 Change 提供模型与兜底 UX，并作为 CHG-029 的前置依赖。

## 影响与边界

### 修改范围

| 层级 | 路径 | 改动内容 |
|------|------|----------|
| Contract | `contracts/schema/v1/mail/dto/mail_message.schema.json` | `direction`: `outbound` \| `inbound` |
| Contract | `contracts/schema/v1/mail/dto/inbound_record.schema.json` | 手动录入 DTO |
| Contract | `contracts/schema/v1/mail/ipc/record_inbound.request.schema.json` | customer_id + subject + from + received_at + body |
| Contract | `contracts/schema/v1/mail/ipc/record_inbound.response.schema.json` | message_id |
| Contract | `contracts/schema/v1/customer/dto/timeline_entry.schema.json` | `email_received` payload |
| Storage | Migration | `mail_message.direction`, `received_at`；`customer_id` 手动录入时必填 |
| Rust | `crates/mail/src/app/record_inbound.rs` | 手动录入 UseCase |
| Rust | `crates/customer/src/app/timeline.rs` | `email_received` |
| Frontend | `apps/desktop/src/features/mail/record-inbound-dialog.tsx` | 录入表单 |
| Frontend | `apps/desktop/src/features/customer/customer-detail.tsx` | 「手动记录回复」入口 |

### 不修改范围

| 路径 | 原因 |
|------|------|
| `crates/mail-net/src/imap.rs` | CHG-029 |
| Python | 无 AI 写库 |

## 依赖关系

- 前置：CHG-013、CHG-015
- 阻塞：CHG-029（IMAP 复用本数据模型）

## 实施方案

1. 扩展 `mail_message`：`direction`、`received_at`。
2. `mail.record_inbound`：手动路径，必填 `customer_id`。
3. 客户详情「手动记录回复」；文案说明优先使用 IMAP 自动同步。

## 验收

- [ ] 可手动录入入站邮件并出现在 timeline
- [ ] `email_received` 与 `email_sent` 可区分
- [ ] CHG-029 可复用同一 `mail_message` 入站结构

## 实际结果

（完成前留空。）

## 后续项

- 见 CHG-029 IMAP 自动收信
