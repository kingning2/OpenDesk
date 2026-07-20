---
id: CHG-20260720-014-crawler-lead-import
title: 爬虫结果导入客户
type: change
status: proposed
priority: P0
owner: developer
domain: crawler
parent: EPIC-20260720-001-mvp-sales-workbench
depends_on:
  - CHG-20260720-013-customer-profile-model
blocks: []
milestone: M1
created: 2026-07-20
updated: 2026-07-20
contracts: crawler import IPC（可选）
related:
  - CHG-20260720-031-crawler-playwright-email-enrich
---

# 爬虫结果导入客户

## 目标

YouTube 爬虫结果页支持 **一键导入为客户**：写入 Customer 表，带来源元数据，邮箱去重；UI 显示已导入/重复状态。

## 非目标

- 修改爬虫抓取逻辑或 YouTube API 适配
- 批量自动导入（MVP 先做单行/多选手动导入）
- 导入后自动发邮件

## 背景

爬虫已能抽取 `email` 并展示在 `crawler-page.tsx`。MVP 获客主路径要求邮箱进入客户档案，而非停留在 crawler 结果表。

## 影响与边界

### 修改范围

| 层级 | 路径 | 改动内容 |
|------|------|----------|
| Contract | `contracts/schema/v1/crawler/ipc/import_customer.request.schema.json`（可选） | channel_result_id → customer |
| Contract | `contracts/schema/v1/crawler/ipc/import_customer.response.schema.json` | 返回 customer_id / 重复标记 |
| Rust | `crates/crawler/src/app/import_customer.rs`（新建） | 调用 customer CreateUseCase |
| Rust | `crates/crawler/src/lib.rs` | 注册 import IPC |
| Rust | `crates/customer/src/app/create.rs` | 暴露可被 crawler 调用的内部 API 或共享 UseCase |
| Storage | `crates/storage/src/crawler_db/models.rs` | 可选：`imported_customer_id` 字段 |
| Storage | crawler migration | 若增加导入回写字段 |
| Frontend | `apps/desktop/src/features/crawler/crawler-page.tsx` | 「导入为客户」按钮 |
| Frontend | `apps/desktop/src/features/crawler/use-crawler-job.ts` | import API hook |
| Frontend | `packages/platform/src/ipc/crawler/` | import 方法 |
| Frontend | `apps/desktop/src/i18n/locales/crawler/` | 导入相关文案 |
| Docs | `docs/managed/domains/crawler/README.md` | 更新导入能力状态 |

### 不修改范围

| 路径 | 原因 |
|------|------|
| `python/packages/crawler/**` | 抓取逻辑不变 |
| `crates/mail/**` | 不涉及发信 |
| `apps/desktop/src/features/customer/**` | 仅跳转链接，不改核心表单 |
| YouTube API 适配器 | 不在范围 |

### Contract

- 可选新增 import IPC；亦可用现有 customer.create + 前端组装 source_meta（实施时选最小方案并记录）

### 跨层

- Rust crawler → Rust customer（进程内调用，非 Python）

### 跨 Feature

- Crawler Feature 调用 customer IPC；不 direct import customer 组件逻辑

### 风险

- 邮箱为空的结果须禁用导入按钮；**频道行保留**，由 CHG-031 Worker 补全后可导入
- 重复邮箱须提示并可选跳转已有客户详情

## 依赖关系

- 父任务：EPIC-20260720-001
- 前置任务：CHG-013
- 阻塞：无

## 实施方案

1. 从 crawler 结果行读取 email、channel id、title、url。
2. 调用 Customer Create，`source_channel=youtube`，`source_meta` 填频道 JSON。
3. 邮箱冲突捕获并返回 `duplicate` 状态码。
4. UI：成功 toast + 跳转客户详情；重复显示已有客户链接。
5. 可选：回写 crawler 结果行 `imported_customer_id`。

## 验收

- [ ] 有邮箱的结果可导入并在客户列表可见
- [ ] 重复邮箱不创建第二条客户
- [ ] 客户详情 `source_meta` 含 YouTube 频道信息
- [ ] 无邮箱行不可导入，但 **保留在列表**；CHG-031 补全邮箱后可导入
- [ ] 架构检查通过

## 实际结果

（完成前留空。）

## 后续项

- 无
