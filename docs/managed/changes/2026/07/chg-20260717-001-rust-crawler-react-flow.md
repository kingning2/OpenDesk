---
id: CHG-20260717-001-rust-crawler-react-flow
title: Rust 多线程 crawler 替代 Python sidecar，并用 React Flow 重写监控界面
type: change
status: completed
priority: P0
owner: cursor-agent
domain: crawler
parent: none
depends_on: []
blocks: []
milestone: none
created: 2026-07-17
updated: 2026-07-17
contracts: required
related: []
---

# Rust 多线程 crawler 替代 Python sidecar，并用 React Flow 重写监控界面

## 目标

1. 将 crawler 执行逻辑从 Python sidecar 迁移到 Rust，本地直接并发执行 YouTube API 抓取。
2. 前端 crawler 页面改为 React Flow 进度监控界面，保留 API Key、CSV、批次选择与启动/停止操作。
3. 保持现有 IPC 能力（start / cancel / status / logs / keywords_import / keywords_batches）对前端可用。

## 非目标

- 不在本次引入第二个平台适配器。
- 不做 React Flow 工作流拖拽编排，仅做执行过程的可视化监控。
- 不做 API Key 安全存储或账号体系集成。

## 实际结果

- 新增 `crates/crawler`：内存任务表、最多 4 worker 线程、YouTube Data API v3 抓取与配额停。
- `crates/app` crawler IPC 改为直连 `CrawlerService`；移除 Rust/Python crawler sidecar 路由与 gateway。
- 删除 `python/packages/crawler` 及 sidecar/gateway crawler handler。
- 前端安装 `@xyflow/react`，`crawler-page` 改为固定四节点进度监控 + 过程日志面板。
- 清理误生成目录：`subscription/`、`tooling/strawberry-perl/`、license 相关未跟踪变更记录。
- 验证：`pnpm lint:types`、crawler 文件 `eslint`、`check_boundary`、`check_contracts` 通过；本机缺 Rust MSVC/GNU 工具链，未完整 `cargo check`。

## 后续项

- 契约层删除遗留 `crawler/sidecar` schema（当前仅运行时不再使用）。
- 进度推送可改为 Tauri event，减少轮询。
