---
id: CHG-20260720-013-customer-profile-model
title: 客户档案模型与详情页
type: change
status: proposed
priority: P0
owner: developer
domain: customer
parent: EPIC-20260720-001-mvp-sales-workbench
depends_on: []
blocks:
  - CHG-20260720-014-crawler-lead-import
  - CHG-20260720-015-smtp-mail-send
milestone: M1
created: 2026-07-20
updated: 2026-07-20
contracts: customer DTO + IPC
related:
  - ADR-0001-ai-readonly-query-port
---

# 客户档案模型与详情页

## 目标

建立 Customer 领域 MVP 数据模型与桌面端基础 UI：客户列表、详情页、新建/编辑（含正式合作字段 B），为爬虫导入与邮件发信提供 `customer_id`。

完成后可观察到的结果：

- SQLite 存在 customer / quote_history / customer_timeline 表（cooperation 字段在主表）
- 桌面端可浏览客户列表、打开详情、手动新建客户
- 邮箱唯一约束生效

## 非目标

- 爬虫导入（CHG-014）
- SMTP 发信（CHG-015）
- AI 只读工具（CHG-017）
- WhatsApp 绑定（CHG-020）
- 报价变更历史 UI（CHG-019，本 Change 只建表结构）

## 背景

MVP 要求 AI 与商务流程都围绕「客户档案」运转。当前仓库无 CRM 表与 UI，仅有 user/tenant 骨架。

## 影响与边界

### 修改范围

| 层级 | 路径 | 改动内容 |
|------|------|----------|
| Contract | `contracts/schema/v1/customer/**` | DTO + IPC schema |
| Storage | `crates/storage/migrations/opendesk/*_create_customer_tables/` | 见 database-schema §5.1 |
| Storage | `crates/storage/src/opendesk_db/` | Diesel models + Repository |
| Rust | `crates/customer/`（新建 crate） | domain/app/infra 分层 |
| Rust | `crates/customer/src/domain/` | Customer 实体、Repository trait |
| Rust | `crates/customer/src/app/` | List/Get/Create/Update UseCase |
| Rust | `crates/customer/src/infra/` | SQLite Repository 实现 |
| Rust | `crates/app/` | 注册 customer IPC commands |
| Rust | `crates/app/Cargo.toml` | 依赖 customer crate |
| Rust | `Cargo.toml` workspace | 成员 customer |
| Frontend | `apps/desktop/src/features/customer/`（新建） | 列表页、详情页、表单 |
| Frontend | `apps/desktop/src/app/routes/` | 注册 customer 路由 |
| Frontend | `apps/desktop/src/i18n/locales/` | customer 文案键 |
| Frontend | `packages/platform/src/ipc/customer/`（新建） | IPC 封装 |
| Docs | `docs/managed/domains/customer/README.md` | 完成后更新为已实现状态 |

### 不修改范围

| 路径 | 原因 |
|------|------|
| `crates/crawler/**` | 导入逻辑在 CHG-014 |
| `crates/mail/**` | 邮件在 CHG-015 |
| `python/**` | 本阶段无 AI 查库 |
| `apps/desktop/src/features/crawler/**` | 仅 CHG-014 增加导入按钮 |
| `contracts/schema/v1/agent/**` | Agent 在 CHG-017 |
| `docs/architecture/product-architecture.md` | 单独 Change 修正叙事 |

### Contract

- **需要**新增 v1 customer IPC/DTO schema
- 枚举值与 Domain README 保持一致

### 跨层

- 是：Contract → Rust IPC → React Feature

### 跨 Feature

- 否（本 Change 不 import 其他 Feature；其他 Feature 后续引用 customer IPC）

### 风险

- `customer` crate 与现有 `user`/`tenant` 命名重叠 → 实施时明确 customer=商务潜客，user=登录用户
- 邮箱唯一冲突需友好错误码

## 依赖关系

- 父任务：[EPIC-20260720-001-mvp-sales-workbench](epic-20260720-001-mvp-sales-workbench.md)
- 前置任务：无
- 阻塞：CHG-014、CHG-015

## 实施方案

1. 按 [`database-schema.md §5.1`](../../../architecture/database-schema.md) 在 `opendesk.db` 创建 customer 相关 Migration。
2. 创建 Contract schema，运行 codegen。
3. 新建 `crates/customer`，实现 Repository + UseCase + IPC handler。
4. 新建 React Feature：列表（分页/搜索邮箱）、详情（展示全部商务字段）、新建/编辑表单。
5. 合作字段 B：`package_name`、`monthly_fee`、`contract_start`、`contract_end`、`cooperation_status` 均在编辑表单可维护。
6. 运行 `check_architecture.py` 与 lint。

## 验收

- [ ] Migration 可应用且可回滚
- [ ] 邮箱重复创建返回明确错误
- [ ] 列表/详情/新建/编辑 UI 可用
- [ ] 合作字段 B 可保存并在详情展示
- [ ] Contract codegen 三端一致
- [ ] `python skills/opendesk/scripts/check_architecture.py` 通过
- [ ] Domain README 更新当前状态
- [ ] 实际结果已回填

## 实际结果

（完成前留空。）

## 后续项

- [CHG-014 爬虫导入](chg-20260720-014-crawler-lead-import.md)
