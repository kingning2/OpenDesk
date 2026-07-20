---
id: CHG-20260720-029-imap-inbound-sync
title: IMAP 自动收信与客户匹配
type: change
status: proposed
priority: P0
owner: developer
domain: mail
parent: EPIC-20260720-001-mvp-sales-workbench
depends_on:
  - CHG-20260720-013-customer-profile-model
  - CHG-20260720-015-smtp-mail-send
  - CHG-20260720-026-mail-inbound-reply-record
  - CHG-20260720-023-opendesk-db-worker-skeleton
blocks:
  - CHG-20260720-018-ai-mail-draft
milestone: M2
created: 2026-07-20
updated: 2026-07-20
contracts: mail imap IPC + sync events
related:
  - ADR-0002-heavy-work-worker-process
---

# IMAP 自动收信与客户匹配

## 目标

MVP **必须**支持 IMAP 自动收取客户回复邮件，写入 `mail_message`（`direction=inbound`）与 `customer_timeline`（`email_received`）。用户已确认：**IMAP 自动收信一定要有**。

完成后可观察到的结果：

- `mail_account` 可配置 IMAP（主机/端口/TLS），与 SMTP 共用同一账号凭据
- Worker 定时轮询 `INBOX`（`background_job.job_type=imap_sync`），**不阻塞 UI**
- 按发件人邮箱自动匹配 `customer.email`；未匹配进入「待关联收件箱」
- `Message-ID` 幂等去重，不重复入库
- 桌面可手动「立即同步」；同步进度/新邮件 Event 通知 UI
- CHG-026 手动录入保留，作为 IMAP 未匹配或异常时的兜底

## 非目标

- 多文件夹全量同步（MVP 仅 `INBOX`）
- 双向已读状态回写 IMAP（`\\Seen` 可选后补）
- 大附件自动下载（MVP 仅存元数据 + 正文；附件后补）
- AI 自动改客户字段
- POP3（仅 IMAP）

## 背景

商务谈价依赖双向邮件。仅 SMTP 发信 + 手动粘贴不够；须自动拉取客户回复供时间线与 AI 上下文使用。IMAP 网络 IO 与解析放在 **Worker**（ADR-0002），主进程只入队与读结果。

## 影响与边界

### 修改范围

| 层级 | 路径 | 改动内容 |
|------|------|----------|
| Contract | `contracts/schema/v1/mail/dto/mail_account.schema.json` | `imap_host`, `imap_port`, `imap_use_tls`, `imap_sync_enabled` |
| Contract | `contracts/schema/v1/mail/dto/mail_message.schema.json` | `direction`, `received_at`, `imap_uid`, `rfc_message_id`, `in_reply_to` |
| Contract | `contracts/schema/v1/mail/ipc/sync_now.request.schema.json` | 触发立即同步 |
| Contract | `contracts/schema/v1/mail/ipc/sync_status.response.schema.json` | 最近同步时间/错误 |
| Contract | `contracts/schema/v1/mail/ipc/inbox_unmatched_list.response.schema.json` | 待关联列表 |
| Contract | `contracts/schema/v1/mail/ipc/link_inbound_customer.request.schema.json` | 手动关联 customer_id |
| Contract | `contracts/schema/v1/mail/events/inbound_received.schema.json` | 新入站事件 |
| Storage | Migration | `mail_account` IMAP 列；`mail_sync_state`；`mail_message` 扩展；`rfc_message_id` UNIQUE |
| Rust | `crates/mail-net/src/imap.rs`（新建） | async-imap 或等价客户端 |
| Rust | `crates/mail/src/app/schedule_imap_sync.rs` | 主进程入队 |
| Rust | `crates/mail/src/app/match_customer.rs` | 发件人 → customer.email |
| Rust | `crates/mail/src/app/link_inbound.rs` | 手动关联 |
| Worker | `crates/worker/src/handlers/imap_sync.rs` | 拉取 UID > last_uid → 解析 → 写 DB |
| Rust | `crates/customer/src/app/timeline.rs` | 匹配成功写 `email_received` |
| Rust | `crates/app/` | mail sync IPC + 默认定时调度（如 3 分钟，可配置） |
| Frontend | `apps/desktop/src/features/mail/mail-settings-panel.tsx` | IMAP 配置区 |
| Frontend | `apps/desktop/src/features/mail/mail-inbox-panel.tsx`（新建） | 待关联 + 最近入站 |
| Frontend | `apps/desktop/src/features/mail/use-mail-sync.ts` | 同步状态 hook |
| Docs | `docs/managed/domains/mail/README.md` | IMAP 说明 |
| Docs | `docs/architecture/database-schema.md` | 表 DDL 更新 |

### 不修改范围

| 路径 | 原因 |
|------|------|
| Python IMAP | 禁止；邮件只在 Rust |
| SMTP 发信逻辑 | CHG-015 已覆盖 |

### Contract

- **需要** IMAP 配置 + sync + unmatched 关联 schema

### 跨层

- Worker IMAP → SQLite → Tauri Event → React
- 主进程 `mail.sync_now` → 入队 `background_job`

### 风险

- Gmail/Outlook 应用专用密码、OAuth2：MVP 文档说明；首版支持用户名+密码/App Password
- 时区与 `received_at`：存 ISO-8601 UTC，UI 本地化
- `customer_id` 可空：未匹配邮件不进客户 timeline，直到手动关联

## 依赖关系

- 父任务：EPIC-20260720-001
- 前置：CHG-013、CHG-015、CHG-026（入站数据模型）、CHG-023（Worker 队列）
- 阻塞：CHG-018（AI 需读入站上下文）

## 实施方案

1. 扩展 `mail_account`：IMAP 字段；`imap_sync_enabled` 默认 true（配置 IMAP 后）。
2. `mail_sync_state`：每账号 `last_uid`、`last_sync_at`、`last_error`。
3. Worker `imap_sync` handler：
   - 连接 IMAP → SELECT INBOX → SEARCH UID `last_uid+1:*`
   - 解析 ENVELOPE/BODY[] → 提取 Message-ID、From、Subject、Date、text/plain
   - INSERT `mail_message` ON CONFLICT(`rfc_message_id`) DO NOTHING
   - 匹配 `customer.email`（大小写不敏感）→ 设 `customer_id` + timeline
4. 主进程：应用启动后注册周期 job；设置页「立即同步」。
5. UI：邮件页「收件」Tab — 已匹配入站列表 + 「待关联」队列 + 关联到客户。
6. CHG-026 手动录入入口保留在客户详情。

## 验收

- [ ] 配置 IMAP 后 Worker 可拉取新邮件且 UI 不卡顿
- [ ] 发件人邮箱匹配已知客户时自动出现 `email_received` timeline
- [ ] 同一 Message-ID 不重复入库
- [ ] 未匹配邮件可手动关联客户
- [ ] `customer.timeline` 只读工具含 IMAP 入站条目
- [ ] 同步失败有 `last_error` 且设置页可见
- [ ] 架构检查通过

## 实际结果

（完成前留空。）

## 后续项

- OAuth2（Gmail/Microsoft）
- 附件下载与 OCR 联动
- 多文件夹、已读回写
