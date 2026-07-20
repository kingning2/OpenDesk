# Mail Domain

## 职责

通过 **用户自有 SMTP** 发送商务邮件，并将发信记录关联到客户时间线。

- SMTP 账号配置（凭据安全存储）
- **邮件模板**管理（内置 + 自定义）、变量渲染
- 撰写邮件：**先选模板 → 填充变量 → 编辑 → 发送**
- 发送、重试、失败原因
- 发件本地记录（含 `template_id`、主题、正文、发送状态）
- **客户邮件回复**：**IMAP 自动收信**（[CHG-029](../../changes/2026/07/chg-20260720-029-imap-inbound-sync.md)）+ **手动录入兜底**（[CHG-026](../../changes/2026/07/chg-20260720-026-mail-inbound-reply-record.md)）
- 为 AI 邮件润色提供已填充模板正文作为输入（发送仍由本领域执行）

## 非职责

- POP3（MVP 仅 IMAP）
- 客户档案主数据（Customer 领域）
- AI 生成/润色逻辑（Agent 领域；本领域提供模板渲染结果）
- 第三方模板市场、群发营销
- Python 直连 SMTP

## 稳定边界

```text
React（模板管理 / 选模板 / 写信 / 发送 UI）
  → Tauri IPC（mail/*）
  → Rust mail UseCase
      ├── TemplateRepository（CRUD）
      ├── TemplateRenderer（{{变量}} ← customer + pricing + sender）
      └── SendMail → mail-net（SMTP）
  → 写入 mail_message + customer_timeline
```

**硬规则：**

- 只有 Rust `mail-net` 接触 SMTP 服务器
- **变量填充在 Rust 完成**，不交给 Python 拼字符串
- 发送必须带 `customer_id`；须记录 `template_id`
- AI 产出为润色草稿；**发送按钮仅人工触发**

## 入口

| 类型 | 路径 |
|------|------|
| Rust crate | `crates/mail/`（骨架）, `crates/mail-net/`（骨架） |
| Contract | `contracts/schema/v1/mail/`（待建） |
| React Feature | `apps/desktop/src/features/mail/`（当前为占位页） |
| Epic 子任务 | [CHG-015](../../changes/2026/07/chg-20260720-015-smtp-mail-send.md)、[CHG-026](../../changes/2026/07/chg-20260720-026-mail-inbound-reply-record.md)、[CHG-029](../../changes/2026/07/chg-20260720-029-imap-inbound-sync.md) |

## 数据模型（MVP 目标）

### `mail_template`

| 字段 | 说明 |
|------|------|
| `id` | 主键 |
| `name` | 显示名，如「首次联系-英文」 |
| `template_intent` | `first_contact` / `follow_up` / `quote_proposal` / `quote_revision` / `cooperation_confirm` / `custom` |
| `subject_template` | 主题模板，含 `{{变量}}` |
| `body_text_template` | 正文纯文本模板 |
| `body_html_template` | 可选 HTML 正文 |
| `locale` | 可选，如 `en` / `zh-cn` |
| `is_system` | 内置模板（不可删，可复制后改） |
| `is_active` | 是否启用 |
| `sort_order` | 列表排序 |

MVP 启动时 Rust migration **种子内置模板**（至少覆盖各 `template_intent` 各 1 份）。

### `mail_account`

| 字段 | 说明 |
|------|------|
| `id` | 主键 |
| `label` | 显示名 |
| `smtp_host` / `smtp_port` | 服务器 |
| `username` | 登录名 |
| `password_ref` | 系统安全存储引用，非明文 |
| `from_address` | 发件人地址 |
| `from_name` | 发件人称呼（供 `{{sender.name}}`） |
| `use_tls` | 是否 TLS |

### `mail_message`

| 字段 | 说明 |
|------|------|
| `id` | 主键 |
| `customer_id` | 外键 |
| `template_id` | 使用的模板（可空=完全手写，MVP 建议必填） |
| `account_id` | 使用的 SMTP 账号 |
| `subject` / `body_text` / `body_html` | **渲染后最终**发送内容 |
| `status` | `draft` / `sending` / `sent` / `failed` |
| `direction` | `outbound` / `inbound`（CHG-026） |
| `received_at` | 入站收到时间（inbound 时） |
| `error_message` | 失败原因 |
| `sent_at` | 发送成功时间 |
| `created_at` | 创建时间 |

## 模板变量（Rust 渲染）

| 变量 | 来源 |
|------|------|
| `{{customer.display_name}}` | customer |
| `{{customer.email}}` | customer |
| `{{customer.source_title}}` | customer.source_meta |
| `{{customer.quoted_price}}` | customer |
| `{{customer.currency}}` | customer |
| `{{customer.package_name}}` | customer |
| `{{customer.pricing_tier}}` | customer |
| `{{pricing.matched_summary}}` | pricing.match（可选） |
| `{{sender.name}}` / `{{sender.email}}` | mail_account |
| `{{today.date}}` | 本地日期 |

缺失变量：**保留占位符或留空**（实施时定一种策略并在 UI 提示缺字段）。

## 当前状态

- `crates/mail`、`crates/mail-net` 仅有 scaffold 注释
- `apps/desktop/src/features/mail/mail-page.tsx` 为占位页
- 无 mail 相关 Contract、无模板表

## 当前约束

- MVP 须 **模板 + 发信 + IMAP 收信 + 手动兜底** 同时交付（M2）
- 凭据不得写入普通日志或 Contract payload 明文传输
- 依赖 Customer 领域 M1 完成后才能渲染客户变量
