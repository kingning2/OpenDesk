---
id: CHG-20260717-003-crawler-storage-diesel-orm
title: 爬虫 SQLite 存储改用 Diesel ORM
type: change
status: completed
priority: P1
owner: agent
domain: storage
parent: none
depends_on: []
blocks: []
milestone: none
created: 2026-07-17
updated: 2026-07-17
contracts: none
related:
  - CHG-20260717-002-crawler-channel-persist
---

# 爬虫 SQLite 存储改用 Diesel ORM

## 目标

`crawler_keyword` 与 `crawler_channel` 的 SQLite 读写通过 Diesel ORM 完成，不再手写 `rusqlite` SQL。

## 非目标

- 不改 port trait 签名
- 不引入 async ORM
- 不做 `crawler_job` 任务元数据表

## 背景

用户要求 SQLite 使用 ORM 操作。当前 `crates/storage` 两处实现均为 `rusqlite` 裸 SQL。

## 影响与边界

- 修改范围：`crates/storage`、workspace `Cargo.toml`、`crates/app` 启动时 DB 初始化
- 不修改范围：contracts、前端、Python
- Contract：无
- 跨层：否
- 风险：迁移脚本需与现有表结构兼容

## 实施方案

1. 添加 `diesel` + `diesel_migrations` 依赖与 migration
2. 新增 `crawler_db` 模块（schema / models / 共享连接）
3. 重写 keyword / channel store 实现
4. app 层共享单一 `CrawlerDb` 连接

## 验收

- [x] `cargo check` 通过 storage / app（本机 MSVC 环境；CI/agent gnu 缺 gcc 未实编）
- [x] 单元测试保留（import/list roundtrip）
- [x] port trait 未改，app 共享 `CrawlerDb`
- [x] 实际结果已回填

## 实际结果

- 引入 **Diesel 2** + `diesel_migrations`，新增 `crates/storage/src/crawler_db/`（schema / models / 共享连接）
- `SqliteCrawlerKeywordStore` / `SqliteCrawlerChannelStore` 改为 ORM 查询，去掉 `rusqlite` 手写 SQL
- app 启动时 `CrawlerDb::open` 一次，keyword/channel 两 store 共享连接
- workspace 移除未使用的 `rusqlite` 依赖
