---
id: CHG-20260716-009-crawler-keywords-csv-db
title: Crawler 关键词 CSV 导入与 SQLite 表
type: change
status: completed
priority: P0
owner: cursor-agent
domain: desktop-runtime
parent: CHG-20260716-008-youtube-api-adapter
depends_on:
  - CHG-20260716-008-youtube-api-adapter
blocks: []
milestone: none
created: 2026-07-16
updated: 2026-07-16
contracts: required
related: []
---

# Crawler 关键词 CSV 导入与 SQLite 表

## 目标

对标 kol-nest-server：关键词通过 **CSV 导入** 写入 **Rust SQLite 表**；启动爬取时按 `batch_id` 读取 `enabled=true` 的关键词，不再手填逗号分隔文本。

## 非目标

- 不做 Excel/xlsx（后续 Change）
- Python 不读库；Rust 解析 batch 后把 `keywords` 逗号串下发 Sidecar
- 不做 crawl progress 断点表（kol 的 `kol_channel_progress` 后续 Change）

## kol 对标

| kol | OpenDesk |
|-----|----------|
| `kol_keyword` (batch_id, text, enabled) | `crawler_keyword` |
| `POST /keywords/import` CSV | IPC `crawler_keywords_import` |
| `GET /keywords/batches` | IPC `crawler_keywords_batches` |
| crawl `batch_id` → DB keywords | `crawler_job_start` + `batch_id` |

## 验收

- [x] CSV 导入返回 batch_id 与 inserted/skipped 统计
- [x] SQLite 表 UNIQUE(batch_id, text)
- [x] 选 batch 启动爬取，Rust 解析 keywords 下发 Sidecar
- [x] Contract + codegen
- [ ] 本地 `pnpm tauri dev` 端到端（需 MSVC 工具链编译 rusqlite）

## 实际结果

- 契约：`keywords.import` / `keywords.batches` IPC；`job.start` 的 `keywords` 改为可选
- Rust：`crawler_keyword` 表（`storage::crawler_keywords::SqliteCrawlerKeywordStore`），CSV 解析对齐 kol（text/enabled 列、去重、255 上限）
- Tauri：`crawler_keywords_import`、`crawler_keywords_batches`；`crawler_job_start` 按 `batch_id` 从 DB 拼逗号串
- 前端：CSV 文件选择 + 批次下拉，移除手填关键词
