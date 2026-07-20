---
id: CHG-20260717-002-crawler-channel-persist
title: crawler 频道结果 SQLite 落库与 job.results IPC
type: change
status: completed
priority: P0
owner: cursor-agent
domain: crawler
parent: CHG-20260717-001-rust-crawler-react-flow
depends_on:
  - CHG-20260717-001-rust-crawler-react-flow
blocks: []
milestone: none
created: 2026-07-17
updated: 2026-07-17
contracts: required
related: []
---

# crawler 频道结果 SQLite 落库与 job.results IPC

## 目标

accepted 频道写入 `crawler_channel` 表，并提供 IPC 查询供 UI 展示。

## 实际结果

- SQLite 表 `crawler_channel`（`SqliteCrawlerChannelStore` + `CrawlerChannelStore` port）
- 爬取 accepted 时映射为 `ChannelRecord` 落库
- 契约 `job.results` IPC + `crawler_job_results` Tauri command
- 前端轮询 `crawlerJobResults` 并展示「收录频道」表
- 修复 `crates/crawler` worker spawn 的 `E0382` 借用错误

## 验证

- `pnpm lint:types` 通过
- `check_contracts` 通过
- `tauri dev` 需在 MSVC 工具链下验证（用户环境已能编译至 crawler crate）
