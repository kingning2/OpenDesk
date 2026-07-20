# Crawler Domain

## 职责

从 **YouTube** 获取潜客线索，重点是频道简介中的 **邮箱** 及频道元数据，供导入客户档案。

- YouTube 关键词/批次爬虫任务
- 频道结果持久化（含 **两阶段邮箱获取**）
- 结果列表 UI 与任务监控
- **导入客户**（去重、合并、来源标记）

## 非职责

- 客户商务字段维护（Customer 领域）
- 非 YouTube 平台获客
- 自动发邮件或自动建 WA 会话
- Python 直连业务 SQLite（爬虫侧car 任务状态除外，已有 crawler 存储边界）

## 稳定边界

```text
React（crawler 页）
  → IPC crawler/*
  → Rust crawler + storage（crawler_db）
  → Python sidecar（YouTube API 调用）
  → 导入动作 → Customer UseCase（email 去重）
```

**已有实现（相对完整）：**

- Contract：`contracts/schema/v1/crawler/**`
- Rust：`crates/crawler/`
- React：`apps/desktop/src/features/crawler/`
- 邮箱抽取：`crates/crawler/src/lib.rs` 内 `extract_email`

**MVP 增量：**

- **两阶段邮箱**（[CHG-031](../../changes/2026/07/chg-20260720-031-crawler-playwright-email-enrich.md)）：
  1. 阶段 1：YouTube API `description` → `extract_email`（已有）
  2. 阶段 2：无邮箱 → Worker Playwright 打开 About 页补全；**不丢弃**频道行
- 爬虫结果行 → `customer` 建档 API（[CHG-014](../../changes/2026/07/chg-20260720-014-crawler-lead-import.md)）
- UI「导入为客户」按钮与邮箱状态 / 重试补全

## 入口

| 类型 | 路径 |
|------|------|
| Rust | `crates/crawler/`, `crates/storage/src/crawler_db/` |
| Contract | `contracts/schema/v1/crawler/` |
| React | `apps/desktop/src/features/crawler/` |
| Epic 子任务 | [CHG-014](../../changes/2026/07/chg-20260720-014-crawler-lead-import.md)、[CHG-031](../../changes/2026/07/chg-20260720-031-crawler-playwright-email-enrich.md) |

## 当前状态

YouTube 爬虫垂直切片 **已可用**（任务启停、日志、频道结果、邮箱展示）。

**尚未实现：** Playwright 邮箱补全、一键导入客户、导入后回写线索状态（已建档/重复）。

## 当前约束

- MVP 获客渠道 **仅 YouTube**
- 导入客户时 `source_channel=youtube`，`source_meta` 须含 channel id/url/title
- 邮箱为空的结果 **保留在列表**，经 CHG-031 Worker 补全；补全前及 `not_found` 状态 **不允许导入**
