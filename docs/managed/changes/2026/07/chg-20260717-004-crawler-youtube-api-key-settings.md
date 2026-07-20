---
id: CHG-20260717-004-crawler-youtube-api-key-settings
title: YouTube API 密钥设置页持久化
type: change
status: completed
priority: P1
owner: agent
domain: crawler
parent: none
depends_on: []
blocks: []
milestone: none
created: 2026-07-17
updated: 2026-07-17
contracts: none
related: []
---

# YouTube API 密钥设置页持久化

## 目标

YouTube API 密钥在桌面端 SQLite 持久化保存，通过标题栏「设置」页面配置；爬虫页不再内联输入密钥。

## 非目标

- 不改动 Python / Contract 跨端契约
- 不做密钥加密（后续可接系统 keychain）

## 影响与边界

- 修改范围：Rust storage/app、platform IPC、desktop settings UI、crawler UI
- Contract：无
- 跨层：否（仅 React → Rust）

## 验收

- [x] 设置页可保存/读取 YouTube API 密钥
- [x] 重启应用后密钥仍可用（SQLite `crawler_setting`）
- [x] 爬虫页从持久化配置启动任务
- [x] 桌面端 TypeScript 检查通过

## 实际结果

- 新增 `crawler_setting` 表与 `CrawlerSettingsStore` 端口；Tauri 命令 `crawler_youtube_api_key_get/set`
- 标题栏「设置」进入 `/settings` 配置密钥；爬虫页移除内联输入，未配置时提示跳转设置
- `pnpm exec tsc --noEmit`（apps/desktop）通过；Rust 单测因本机缺少 gcc/dlltool 未执行
