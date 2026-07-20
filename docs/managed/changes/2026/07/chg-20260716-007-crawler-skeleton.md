---
id: CHG-20260716-007-crawler-skeleton
title: 多平台爬虫骨架（YouTube 先行）+ Contract + 过程日志
type: change
status: completed
priority: P1
owner: cursor-agent
domain: python-runtime
parent: none
depends_on: []
blocks: []
milestone: none
created: 2026-07-16
updated: 2026-07-16
contracts: required
related: []
---

# 多平台爬虫骨架（YouTube 先行）+ Contract + 过程日志

## 目标

1. **Contract 先行**：定义平台无关的爬虫 Job / 进度 / 过程日志 / 结果契约；`platform` 枚举可扩展（首版仅 `youtube`）。
2. **Python Skeleton**：新建 `python/packages/crawler`，`PlatformAdapter` Port + YouTube Mock；经 gateway/queue/worker 调度。
3. **过程日志可见**：爬取每一步产出结构化过程日志，经两条通道到达桌面侧（见下），用户能看到「正在搜关键词 / 过滤频道 / 配额」等过程。

## 已确认范围（2026-07-16）

| 项 | 结论 |
|---|---|
| 首发平台 | 仅 YouTube Data API |
| 后续平台 | 契约与 Port 预留 `platform`，不写死 YouTube |
| Contract | **必须做**（本 Change 内） |
| 过程日志 | **必须可见**（领域事件 + runtime 结构化日志） |
| UI / Rust IPC 接线 | 本 Change **不做**桌面 UI；契约与 Python 侧发射就绪，Rust/React 另开 `frontend/*` |

## 非目标

- 不迁移 kol-nest MySQL / Redis / Nest / Excel。
- Python 不持久化业务状态；密钥由 Rust 注入。
- 不做 DOM 爬虫；不在本 Change 实现真实 YouTube HTTP（仅 Port + Mock；真 Adapter 后续 Change）。
- 不改 `apps/desktop/**`、`crates/**`（除非 codegen 生成物落在允许路径且脚本自动生成——本分支禁止 crates，故 codegen 后的 Rust 引用留给 frontend/rust 分支）。

## 过程日志设计（双通道）

```text
爬虫步骤
  ├─① domain event: crawler.job.log / crawler.job.progress
  │     → Rust 转发 → React 任务日志面板（产品可见）
  └─② runtime/log/entry/v1（feature=crawler, task_id=...）
        → Sidecar stdout → Rust 日志接管（运维/调试可见）
```

- **① 领域过程日志**：面向「这个 Job 在干什么」，字段含 `job_id`、`seq`、`phase`、`message`、`stats`；UI 按 job 订阅。
- **② Runtime 日志**：复用已有 `runtime/log/entry/v1`，`logger=crawler.*`，带 `task_id`/`event`；不替代 ①。

首版 `phase` 建议：`job_started` | `keyword_begin` | `search_page` | `channel_batch` | `filter` | `quota` | `keyword_done` | `job_completed` | `job_failed`。

## Contract 草案（实现时落 schema）

命名空间：`crawler/`（feature = crawler）

| Kind | Name | 用途 |
|------|------|------|
| dto | `platform` | enum：`youtube`（预留注释/后续加值） |
| dto | `job-config` | keywords、rate_limit_ms、max_total、year、min_year_video_count、exclude_countries… |
| dto | `channel-result` | platform、channel_id、title、country、subscriber_count、email?、… |
| ipc | `job.start` request/response | 启动 Job，返回 `job_id` |
| ipc | `job.cancel` request/response | 取消 |
| ipc | `job.status` request/response | 查询状态 |
| event | `job.started` | 任务开始 |
| event | `job.progress` | 计数/配额/当前关键词 |
| event | `job.log` | **过程日志行**（UI 主消费） |
| event | `job.completed` | 结束 + stop_reason |
| event | `job.failed` | 失败原因 |

所有 payload 带 `platform` + `job_id`，避免日后多平台串台。

## 影响与边界

- 修改范围：
  - `contracts/schema/v1/crawler/**`
  - `contracts/CHANGELOG.md` + sync/codegen（Python 生成物）
  - `python/packages/crawler/**`（新建）
  - `python/packages/gateway` 注册 handler（骨架）
- 不修改：`apps/**`、`crates/**`、kol-nest
- Contract：本 Change 必含；需 PR 评审
- 跨层：Contract → Codegen(Python) → Python Skeleton；Rust/React 后续
- 风险：过程日志频率过高可能刷屏 → Mock/真 Adapter 均需 rate 感知，`job.log` 可合并批量（首版逐条 + seq）

## 实施方案

### 阶段 A — 规划（当前）

- [x] 平台：YouTube 先行、可扩展
- [x] Contract 必做
- [x] 过程日志双通道
- [ ] 负责人确认本草案后 → `approved`

### 阶段 B — Contract（approved 后第一步）

1. `create_contract.py` 生成上表 schema
2. 填字段、`CHANGELOG`、`sync_contracts.py`、`check_contracts.py`

### 阶段 C — Python Skeleton

1. `create_python_package.py --name crawler`
2. `PlatformAdapter` Protocol；`YoutubeMockAdapter` 按 phase 发 `job.log` + progress
3. Gateway handler：start/cancel/status（契约类型）
4. 单测：phase 序列、日志 seq 单调、配额 cost 表（对标 kol-nest calculator）

### 阶段 D — 后续独立 Change

- YouTube 真 Adapter
- Rust IPC 转发 + 密钥注入
- React 任务日志面板订阅 `crawler.job.log`

## 验收

- [x] 范围已确认（本轮用户反馈）
- [x] Contract schema 齐全且 `check_contracts` 通过
- [x] Mock 跑一轮能产出有序 `job.log` 事件（测试断言）
- [x] Runtime 日志带 `feature=crawler` + `task_id`
- [x] 架构边界检查通过；无 Python 持久化/无对外 HTTP
- [x] 实际结果已回填

## 实际结果

- 分支：`python/feature/crawler`
- Contract：`contracts/schema/v1/crawler/**`（dto/ipc/event/sidecar）+ OpenAPI path fragments；`sync_contracts` 已生成三端类型
- Python：`python/packages/crawler`（Port + YoutubeMock + JobService + 配额算法）
- Sidecar：`POST /v1/crawler/job/{start,cancel,status}`
- 过程日志：`crawler.job.log`（seq 单调）+ `shared.logging`（feature=crawler）
- 验证：`pnpm lint:python` 通过；`check_contracts` / `check_boundary` 通过；Mock 烟测 phases 完整
- 附带：去掉各 `python/**/pyproject.toml` UTF-8 BOM（uv/tomllib 无法解析）；`sync_contracts.py` PascalCase 与 Python 生成格式对齐 ruff

## 后续项

- CHG：YouTube real adapter
- CHG：Rust/React 过程日志面板（订阅 `crawler.job.log`）
- CHG：第二平台（改 Contract enum + 新 Adapter）
