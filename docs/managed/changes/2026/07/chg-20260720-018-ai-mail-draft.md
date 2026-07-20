---
id: CHG-20260720-018-ai-mail-draft
title: AI 邮件润色（基于模板，人审后发）
type: change
status: proposed
priority: P0
owner: developer
domain: agent
parent: EPIC-20260720-001-mvp-sales-workbench
depends_on:
  - CHG-20260720-015-smtp-mail-send
  - CHG-20260720-016-pricing-catalog
  - CHG-20260720-017-ai-readonly-query-tools
  - CHG-20260720-027-llm-provider-settings
  - CHG-20260720-029-imap-inbound-sync
blocks: []
milestone: M3
created: 2026-07-20
updated: 2026-07-20
contracts: agent mail draft IPC
related:
  - ADR-0001-ai-readonly-query-port
  - ADR-0005-ai-correction-memory
---

# AI 邮件润色（基于模板，人审后发）

## 目标

在 **已选模板且 Rust 已填充变量** 的邮件编辑器中，AI 根据该客户档案 + 价目表 + 模板正文 **润色/个性化**；用户编辑后 **手动点击发送**（CHG-015）。AI **不能**触发发送，**不能**跳过模板体系从空白生成。

## 非目标

- 从空白邮件起草（M2 须先选 Mail 模板）
- 自动发送、定时发送
- AI 修改客户报价或合作状态
- 多语言邮件（可后续）
- WA 回复建议（CHG-020）

## 背景

用户要求 AI 懂价目表且记得每个客户。生成邮件是 MVP 核心 AI 能力，须在 M2 发信通路就绪后接入。

## 影响与边界

### 修改范围

| 层级 | 路径 | 改动内容 |
|------|------|----------|
| Contract | `contracts/schema/v1/agent/ipc/mail_draft.request.schema.json` | customer_id + **template_id** + intent + 已渲染 subject/body |
| Contract | `contracts/schema/v1/agent/ipc/mail_draft.response.schema.json` | subject + body + disclaimers |
| Contract | `contracts/schema/v1/agent/dto/mail_intent.schema.json` | `first_contact` / `follow_up` / `revise_quote` 等 |
| Rust | `crates/agent/src/app/mail_draft.rs` | 编排：context_loader → sidecar |
| Rust | `crates/app/` | 注册 mail_draft IPC |
| Python | `python/packages/agent/agent/tasks/mail_draft.py`（新建） | Prompt + 生成 |
| Python | `python/packages/agent/agent/prompts/mail_draft/`（新建） | 在模板骨架上润色的 Prompt |
| Frontend | `apps/desktop/src/features/mail/ai-draft-panel.tsx`（新建） | 「AI 润色」生成/插入/重试 |
| Frontend | `apps/desktop/src/features/mail/mail-page.tsx` | 集成 AI 面板 |
| Frontend | `apps/desktop/src/features/mail/use-mail-draft.ts` | hook |
| Frontend | `packages/platform/src/ipc/agent/mail_draft.ts` | IPC |
| Frontend | i18n agent/mail 键 | 文案 |
| Docs | `docs/managed/domains/agent/README.md` | mail_draft 能力 |

### 不修改范围

| 路径 | 原因 |
|------|------|
| `crates/mail-net/src/smtp.rs` | 发送逻辑不变 |
| `crates/customer` 写 UseCase | AI 不改库 |
| 自动发送 UI | 禁止 |
| `crates/channel/**` | WA 不在范围 |

### Contract

- **需要** mail_draft IPC schema

### 跨层

- React → Rust agent → Python sidecar → Rust Query Port（只读）→ Python 生成 → Rust → React

### 跨 Feature

- Mail UI 调用 agent IPC；发送仍走 mail IPC

### 风险

- 生成前必须成功拉取 customer.get；失败则 UI 提示补全客户卡
- Prompt 须注入 cooperation_status、quoted_price、pricing tiers，避免幻觉报价

## 依赖关系

- 父任务：EPIC-20260720-001
- 前置任务：CHG-015、CHG-016、CHG-017
- 阻塞：无

- 须在 `template_apply` 之后调用；无 template_id 拒绝润色

## 实施方案

1. 定义 `mail_intent`（与 `template_intent` 对齐）与 draft Contract。
2. Rust `mail_draft` UseCase：校验 customer_id + template_id → context_loader → 传入 **已渲染模板正文** → Python。
3. Python Prompt：保留模板结构（称呼、报价段、CTA），只润色语气与个性化细节；注入客户 + 价目字段。
4. UI：「AI 润色」按钮（模板 apply 后可用）→ 覆盖/插入编辑器 → 用户改 → 「发送」走 CHG-015。
5. **禁止**「AI 润色并发送」合一按钮。

## 验收

- [ ] 未选模板时 AI 润色不可用
- [ ] 润色结果保留模板结构且引用客户来源与报价
- [ ] 价目表信息出现在建议内容中
- [ ] 发送仅通过 mail send IPC，无 agent 发送路径
- [ ] 缺 customer_id 拒绝生成
- [ ] 架构检查通过

## 实际结果

（完成前留空。）

## 后续项

- 无
