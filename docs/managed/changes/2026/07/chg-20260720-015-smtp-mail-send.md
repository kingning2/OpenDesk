---
id: CHG-20260720-015-smtp-mail-send
title: SMTP 发信、邮件模板与客户时间线
type: change
status: completed
priority: P0
owner: developer
domain: mail
parent: EPIC-20260720-001-mvp-sales-workbench
depends_on:
  - CHG-20260720-013-customer-profile-model
blocks:
  - CHG-20260720-018-ai-mail-draft
milestone: M2
created: 2026-07-20
updated: 2026-07-22
contracts: mail IPC + DTO
related: []
---

# SMTP 发信、邮件模板与客户时间线

## 目标

用户配置自有 SMTP；对客户发信时 **必须先选邮件模板**，Rust 填充客户/价目/发件人变量后进入编辑器，再 **人工发送**；发送结果写入 `mail_message`（含 `template_id`）与客户 `customer_timeline`。

商务邮件不从空白页开始——模板是 M2 必交付能力。

这是 MVP **优先打通** 的主链路（用户已确认）。

## 非目标

- IMAP 收信、收件箱同步
- AI 润色（CHG-018）
- 多 SMTP 账号高级路由
- 空白编辑器直接发信（须选模板或基于模板渲染后的草稿）
- 大附件（MVP 可后补小附件）

## 背景

`mail` / `mail-net` crate 与 mail 页面均为占位。商务谈价依赖邮件发出并留痕。

## 影响与边界

### 修改范围

| 层级 | 路径 | 改动内容 |
|------|------|----------|
| Contract | `contracts/schema/v1/mail/dto/mail_template.schema.json` | 模板 DTO |
| Contract | `contracts/schema/v1/mail/dto/template_intent.schema.json` | 模板类型枚举 |
| Contract | `contracts/schema/v1/mail/ipc/template_list.response.schema.json` | 模板列表 |
| Contract | `contracts/schema/v1/mail/ipc/template_save.request.schema.json` | 新建/编辑自定义模板 |
| Contract | `contracts/schema/v1/mail/ipc/template_apply.request.schema.json` | customer_id + template_id → 渲染结果 |
| Contract | `contracts/schema/v1/mail/ipc/template_apply.response.schema.json` | subject + body 预览 |
| Contract | `contracts/schema/v1/mail/dto/mail_account.schema.json` | SMTP 账号 DTO（不含明文密码） |
| Contract | `contracts/schema/v1/mail/dto/mail_message.schema.json` | 邮件消息 DTO（含 template_id） |
| Contract | `contracts/schema/v1/mail/ipc/account_save.request.schema.json` | 保存 SMTP 配置 |
| Contract | `contracts/schema/v1/mail/ipc/account_list.response.schema.json` | 账号列表 |
| Contract | `contracts/schema/v1/mail/ipc/send.request.schema.json` | customer_id + template_id + subject + body |
| Contract | `contracts/schema/v1/mail/ipc/send.response.schema.json` | message_id + status |
| Contract | `contracts/CHANGELOG.md` | 登记 |
| Rust | `crates/mail-net/src/smtp.rs`（新建） | lettre 或等价 SMTP 客户端 |
| Rust | `crates/mail-net/Cargo.toml` | SMTP 依赖 |
| Rust | `crates/mail/src/domain/mail_template.rs` | 模板实体 |
| Rust | `crates/mail/src/app/template_render.rs` | {{变量}} 渲染（customer + pricing + sender） |
| Rust | `crates/mail/src/app/template_crud.rs` | 列表/保存/内置种子 |
| Rust | `crates/mail/src/domain/` | MailAccount、MailMessage 实体 |
| Rust | `crates/mail/src/app/send_mail.rs` | 发送 UseCase |
| Rust | `crates/mail/src/app/save_account.rs` | 账号配置 UseCase |
| Rust | `crates/mail/src/infra/` | DB + 密钥存储（Tauri secure store 或 OS keychain） |
| Rust | `crates/customer/src/app/timeline.rs` | 发信成功后追加 timeline 条目 |
| Storage | `crates/storage/src/mail_db/`（新建） | mail_template、mail_account、mail_message 表 |
| Storage | `crates/storage/src/mail_db/seeds/` | 内置模板种子数据 |
| Rust | `crates/app/` | 注册 mail IPC |
| Frontend | `apps/desktop/src/features/mail/mail-page.tsx` | 写信 UI：模板选择 → 预览 → 编辑 |
| Frontend | `apps/desktop/src/features/mail/mail-template-manager.tsx`（新建） | 模板管理 CRUD |
| Frontend | `apps/desktop/src/features/mail/use-mail-template.ts` | 列表/apply/save hook |
| Frontend | `apps/desktop/src/features/mail/use-mail-send.ts` | 发送 hook |
| Frontend | `apps/desktop/src/features/mail/mail-settings-panel.tsx`（新建） | SMTP 配置 |
| Frontend | `apps/desktop/src/features/customer/` | 详情页「写邮件」入口 |
| Frontend | `packages/platform/src/ipc/mail/` | IPC 封装 |
| Frontend | `apps/desktop/src/i18n/locales/mail/` | 文案 |
| Contract | `contracts/CHANGELOG.md` | 登记 |
| Docs | `docs/managed/domains/mail/README.md` | 更新状态 |

### 不修改范围

| 路径 | 原因 |
|------|------|
| `python/**` | 发信不经 Python |
| `crates/channel/**` | WhatsApp 在 M5 |
| `contracts/schema/v1/agent/**` | AI 起草 CHG-018 |
| Agent 自动发送逻辑 | 禁止 |

### Contract

- **需要**新增 mail v1 schema

### 跨层

- React → Rust → SMTP 外网；Rust → SQLite

### 跨 Feature

- Mail 调用 customer timeline UseCase（Rust 层，非 React cross-import）

### 风险

- SMTP 凭据存储方式须符合安全规范
- 发送失败须保留 `failed` 状态与 error_message
- 真实 SMTP 测试需测试账号

## 依赖关系

- 父任务：EPIC-20260720-001
- 前置任务：CHG-013
- 阻塞：CHG-018

## 实施方案

1. Migration：`mail_template`、`mail_account`、`mail_message`；内置模板 seeds。
2. 实现 `TemplateRenderer`：从 customer、pricing（可选）、mail_account 解析 `{{变量}}`。
3. 实现 SMTP 发送（TLS、认证、超时）。
4. 密码写入 OS 安全存储，DB 只存 `password_ref`。
5. 发信流程：**选模板 → apply 渲染 → 编辑 → send**；timeline 记录 `template_id`。
6. UI：模板管理页 + 写信页模板下拉；客户详情「写邮件」默认按 `lifecycle_status` 推荐模板 intent。
7. 发送按钮 **仅人工**。

## 验收

- [x] 内置模板可列出并按 customer 渲染变量
- [x] 可新建/编辑自定义模板（custom）
- [x] 配置 SMTP 后可基于模板发出测试邮件（SMTP 经 `mail-net` / lettre；需真实账号人工验证）
- [x] mail_message 保存 template_id
- [x] 发送成功/失败状态正确（`sent` / `failed` + `error_message`）
- [x] 客户时间线可见发信条目（含所用模板名）
- [x] 日志无明文密码（密码进 OS keyring；SMTP 错误脱敏）
- [x] 架构与 lint 通过（`cargo check` MSVC；`tsc` desktop；frontend lint 针对本次改动）

## 实际结果

- `mail-net::send_smtp` + `SendMail` 真发信；失败仍落库并写时间线
- `mail_template_save` Contract/IPC/UI；系统模板只读
- 账号密码写入 keyring（`password_ref=keyring:OpenDesk/mail_account/{id}`），DB 不再存明文
- 时间线 summary：`Email sent|failed [模板名]: 主题`
- 客户详情「写邮件」→ `/features/mail?customerId=`，并按 lifecycle 推荐模板 intent
- `{{today.date}}` 渲染为 UTC `YYYY-MM-DD`

## 后续项

- [CHG-018 AI 邮件起草](chg-20260720-018-ai-mail-draft.md)
