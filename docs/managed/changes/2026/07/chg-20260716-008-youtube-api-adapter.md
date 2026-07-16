---
id: CHG-20260716-008-youtube-api-adapter
title: YouTube 真 API Adapter + 前端传入 api_key
type: change
status: in_progress
priority: P0
owner: cursor-agent
domain: python-runtime
parent: none
depends_on:
  - CHG-20260716-007-crawler-skeleton
blocks: []
milestone: none
created: 2026-07-16
updated: 2026-07-16
contracts: required
related: []
---

# YouTube 真 API Adapter + 前端传入 api_key

## 目标

1. Contract 增加 `api_key`（IPC + Sidecar `job.start` / `job_config`），由前端写入、经 Rust 下发，Python 不读环境变量密钥。
2. 实现 `YoutubeApiAdapter`（YouTube Data API v3），保留 `youtube_mock` 供测试。
3. 前端提供录入 API Key 并启动任务的最小 UI；Rust 透传 IPC → Sidecar。

## 非目标

- 不做密钥云端同步 / OS keychain 持久化（首版可会话内持有或本地轻量存储，若做则明文不落日志）。
- 不迁移 kol-nest 的 DB/进度表/Excel。
- 不把 api_key 打进日志或过程日志 detail。

## 影响与边界

- 修改：`contracts/**`、`python/packages/crawler/**`、codegen；前端分支另含 `apps/desktop`、`packages/platform`、`crates/**`
- Contract：Breaking 增量字段（可选 `api_key`）；youtube 真跑时 required at runtime
- 跨层：Contract → Codegen → Python →（frontend 分支）Rust → React

## 实施方案

1. Schema 增加 `api_key`；CHANGELOG 0.1.6；sync
2. `JobConfig.api_key`；`YoutubeApiAdapter`；registry：`youtube` 真、`youtube_mock` 测
3. 前端分支：IPC + 简单表单（keywords + api_key + start/status/logs 展示）

## 验收

- [ ] `api_key` 出现在 IPC/Sidecar start 契约
- [ ] 真 Adapter 能调 search/channels；quotaExceeded → stop_reason
- [ ] 日志/事件不含 api_key 明文
- [ ] 前端可写入 key 并 start（或本 Change 拆 PR 时 Python 侧先绿）
- [ ] lint / check_contracts 通过

## 实际结果

（实现中）

## 后续项

- 密钥安全存储（keychain）
- 过程日志面板打磨
