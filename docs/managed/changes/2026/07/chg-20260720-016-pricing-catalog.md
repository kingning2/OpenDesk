---
id: CHG-20260720-016-pricing-catalog
title: 价目表与阶梯报价
type: change
status: proposed
priority: P0
owner: developer
domain: pricing
parent: EPIC-20260720-001-mvp-sales-workbench
depends_on:
  - CHG-20260720-013-customer-profile-model
blocks:
  - CHG-20260720-017-ai-readonly-query-tools
  - CHG-20260720-018-ai-mail-draft
milestone: M3
created: 2026-07-20
updated: 2026-07-20
contracts: pricing IPC + DTO
related: []
---

# 价目表与阶梯报价

## 目标

提供企业价目表与阶梯报价的本地维护与查询，使 AI 起草邮件时能引用 **标准套餐与阶梯价格**（用户要求 AI 懂价目表）。

## 非目标

- 客户个体报价变更 UI（CHG-019）
- AI 自动改价
- 复杂计费引擎、发票
- 与外部 ERP 同步

## 背景

谈价邮件需要同时知道「公司标准价」和「该客户当前报价」。本 Change 负责前者。

## 影响与边界

### 修改范围

| 层级 | 路径 | 改动内容 |
|------|------|----------|
| Contract | `contracts/schema/v1/pricing/dto/package.schema.json` | 套餐 DTO |
| Contract | `contracts/schema/v1/pricing/dto/tier.schema.json` | 阶梯 DTO |
| Contract | `contracts/schema/v1/pricing/ipc/list_packages.response.schema.json` | 列表 |
| Contract | `contracts/schema/v1/pricing/ipc/save_package.request.schema.json` | 保存套餐 |
| Contract | `contracts/schema/v1/pricing/ipc/match_tier.request.schema.json` | 阶梯匹配 |
| Contract | `contracts/schema/v1/pricing/ipc/match_tier.response.schema.json` | 匹配结果 |
| Rust | `crates/pricing/`（新建）或 `crates/customer/src/pricing/` | 领域逻辑 |
| Storage | `crates/storage/src/pricing_db/` | pricing_package、pricing_tier 表 |
| Rust | `crates/app/` | pricing IPC |
| Frontend | `apps/desktop/src/features/pricing/` 或 `settings/pricing-tab.tsx` | 管理 UI |
| Frontend | `packages/platform/src/ipc/pricing/` | IPC |
| Frontend | i18n pricing 键 | 文案 |
| Docs | `docs/managed/domains/pricing/README.md` | 更新状态 |

### 不修改范围

| 路径 | 原因 |
|------|------|
| `python/**` | 查价表走 CHG-017 Query Port |
| `customer` 的 `quoted_price` 字段逻辑 | 个体报价在 CHG-019 |
| Mail 发送 | CHG-015 |

### Contract

- **需要** pricing v1 schema

### 跨层

- React → Rust → SQLite

### 跨 Feature

- 否

### 风险

- MVP 阶梯匹配规则宜简单（数量区间），避免过度设计

## 依赖关系

- 父任务：EPIC-20260720-001
- 前置任务：CHG-013
- 阻塞：CHG-017、CHG-018

## 实施方案

1. Migration + Contract + codegen。
2. CRUD 套餐与阶梯；`match_tier` 按数量/套餐 id 返回命中档。
3. UI：表格编辑套餐；每套餐下编辑阶梯行。
4. 可选：JSON/CSV 导入入口（带 Rust 校验）。
5. 种子数据：开发环境示例套餐便于 AI 测试。

## 验收

- [ ] 可创建套餐与多档阶梯
- [ ] `match_tier` 返回正确档位
- [ ] UI 可管理价目表
- [ ] 架构检查通过

## 实际结果

（完成前留空。）

## 后续项

- [CHG-017 AI 只读 Query Port](chg-20260720-017-ai-readonly-query-tools.md)
