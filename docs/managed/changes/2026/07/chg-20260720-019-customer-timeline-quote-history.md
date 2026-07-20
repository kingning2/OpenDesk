---
id: CHG-20260720-019-customer-timeline-quote-history
title: 报价与合作变更审计及时间线
type: change
status: proposed
priority: P1
owner: developer
domain: customer
parent: EPIC-20260720-001-mvp-sales-workbench
depends_on:
  - CHG-20260720-013-customer-profile-model
  - CHG-20260720-015-smtp-mail-send
blocks:
  - CHG-20260720-020-whatsapp-business-assist
milestone: M4
created: 2026-07-20
updated: 2026-07-20
contracts: customer quote/timeline IPC
related: []
---

# 报价与合作变更审计及时间线

## 目标

完善客户 **正式合作字段 B** 的变更体验：报价修改写入 `quote_history`；合作套餐/月费/合约起止变更可审计；客户详情展示 **完整时间线**（邮件/WA/人工备注/报价变更）。

确保 AI 通过 `quote.history` 与 `customer.timeline` 能查到准确历史（依赖 CHG-017 工具）。

## 非目标

- 新建客户基础 CRUD（CHG-013 已做）
- AI 写历史
- 复杂审批流

## 背景

用户要求合作信息为正式字段（套餐、月费、到期），且 AI 必须清楚每个客户状态。仅有主表字段不够，须历史与 timeline。

## 影响与边界

### 修改范围

| 层级 | 路径 | 改动内容 |
|------|------|----------|
| Contract | `contracts/schema/v1/customer/dto/quote_history_entry.schema.json` | 报价历史 DTO |
| Contract | `contracts/schema/v1/customer/dto/timeline_entry.schema.json` | 时间线 DTO |
| Contract | `contracts/schema/v1/customer/ipc/update_quote.request.schema.json` | 改价 + reason |
| Contract | `contracts/schema/v1/customer/ipc/update_cooperation.request.schema.json` | 改合作字段 |
| Contract | `contracts/schema/v1/customer/ipc/add_note.request.schema.json` | 人工备注 |
| Contract | `contracts/schema/v1/customer/ipc/timeline_list.request.schema.json` | 时间线列表 |
| Rust | `crates/customer/src/app/update_quote.rs` | 写 quote_history + 更新主表 |
| Rust | `crates/customer/src/app/update_cooperation.rs` | 合作字段 + audit |
| Rust | `crates/customer/src/app/add_note.rs` | timeline note |
| Rust | `crates/customer/src/query/quote_history.rs` | 供 CHG-017 查询 |
| Rust | `crates/customer/src/query/timeline.rs` | 供 CHG-017 查询 |
| Frontend | `apps/desktop/src/features/customer/customer-detail-page.tsx` | 时间线 Tab |
| Frontend | `apps/desktop/src/features/customer/quote-edit-dialog.tsx` | 改价对话框 |
| Frontend | `apps/desktop/src/features/customer/cooperation-edit-form.tsx` | 合作字段表单 |
| Docs | `docs/managed/domains/customer/README.md` | 更新 |

### 不修改范围

| 路径 | 原因 |
|------|------|
| `python/**` 写路径 | AI 不改库 |
| `crates/mail-net/**` | 发信逻辑不变；timeline 仍由 mail UseCase 触发 |
| Pricing 价目表 CRUD | CHG-016 |

### Contract

- **需要**扩展 customer IPC

### 跨层

- React → Rust 写；AI 只读 query 已在 CHG-017

### 跨 Feature

- 否

### 风险

- 改价须原子：history 插入 + 主表更新同一事务

## 依赖关系

- 父任务：EPIC-20260720-001
- 前置任务：CHG-013、CHG-015（timeline 已有 email_sent 条目）
- 阻塞：CHG-020（WA 消息也写 timeline）

## 实施方案

1. 扩展 Contract 与 UseCase。
2. 改价 UI：旧价→新价、原因、操作人（本地用户）。
3. 合作字段 B 独立编辑区：package_name、monthly_fee、contract_start/end、cooperation_status。
4. 时间线 UI：按时间倒序，类型图标区分 email/wa/note/quote_change。
5. 确认 CHG-017 的 quote.history / customer.timeline 返回新数据。

## 验收

- [ ] 改价产生 quote_history 记录且主表同步
- [ ] 合作字段变更可保存并在详情展示
- [ ] 时间线展示发信 + 备注 + 改价事件
- [ ] AI 只读工具可读到新历史
- [ ] 架构检查通过

## 实际结果

（完成前留空。）

## 后续项

- [CHG-020 WhatsApp Business 辅助](chg-20260720-020-whatsapp-business-assist.md)
