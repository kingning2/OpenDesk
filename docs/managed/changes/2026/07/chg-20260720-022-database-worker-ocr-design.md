---
id: CHG-20260720-022-database-worker-ocr-design
title: 数据库 Schema + Worker 进程 + OCR 设计文档
type: change
status: completed
priority: P0
owner: product-owner
domain: storage
parent: EPIC-20260720-001-mvp-sales-workbench
depends_on:
  - CHG-20260720-021-product-narrative-realignment
blocks:
  - CHG-20260720-023-opendesk-db-worker-skeleton
  - CHG-20260720-024-ocr-worker-pipeline
milestone: M0-infra
created: 2026-07-20
updated: 2026-07-20
contracts: none
related:
  - ADR-0002-heavy-work-worker-process
---

# 数据库 Schema + Worker 进程 + OCR 设计文档

## 目标

定稿并落库以下 **规划文档**（不含业务代码）：

1. **`opendesk.db` 全表设计**（CRM / Mail / Pricing / Channel / OCR / background_job）
2. **进程模型**：OCR 与重任务 **禁止** 在 Tauri UI 主进程执行
3. **Storage / OCR 领域**边界与 ADR-0002

完成后团队可并行：M1–M5 按表实施；OCR 按 Worker 模型实施。

## 非目标

- 不编写 Diesel Migration / Rust 代码
- 不实现 OCR 引擎
- 不改造现有 YouTube 爬虫进程模型（仍 in-process，文档已标注未来可迁 Worker）

## 影响与边界

### 新增/更新文档

| 文件 | 内容 |
|------|------|
| [`docs/architecture/database-schema.md`](../../../architecture/database-schema.md) | SQLite 双库、全表 DDL、ER、AI 可读表 |
| [`docs/architecture/process-model.md`](../../../architecture/process-model.md) | 三进程模型、OCR 时序、UI vs Worker 分工 |
| [`docs/managed/decisions/runtime/adr-0002-heavy-work-worker-process.md`](../../decisions/runtime/adr-0002-heavy-work-worker-process.md) | Worker 长期决策 |
| [`docs/managed/domains/storage/README.md`](../../domains/storage/README.md) | Storage 领域 |
| [`docs/managed/domains/ocr/README.md`](../../domains/ocr/README.md) | OCR 领域 |
| [`docs/managed/domains/runtime/README.md`](../../domains/runtime/README.md) | 运行时/Worker 领域 |
| [`docs/architecture/README.md`](../../../architecture/README.md) | 索引链接 |
| [`docs/managed/roadmaps/mvp-sales-workbench.md`](../../roadmaps/mvp-sales-workbench.md) | M6 Worker+OCR |
| [`docs/managed/MVP_REVIEW.md`](../../MVP_REVIEW.md) | 评审入口补充 |

### 不修改范围

- `crates/**`、`contracts/**`、`python/**` 实现
- `crawler.db` 既有 Migration（仅文档描述 CHG-014 增量字段）

## 核心设计结论

### 双库

| 库 | 用途 |
|----|------|
| `crawler.db` | 爬虫（已有） |
| `opendesk.db` | 商务 CRM + 邮件 + 价目 + 渠道 + OCR + 任务队列 |

### 进程（硬约束）

| 进程 | OCR / 重任务 |
|------|----------------|
| Tauri 主进程 | **禁止** OCR 引擎；仅入队、查状态、读结果 |
| `opendesk-worker` | **必须** 在此执行 OCR |
| Python Sidecar | **禁止** SQLite；可接收已 OCR 文本做 AI |

### OCR 数据流

```text
UI → ocr.submit → 主进程 INSERT background_job + ocr_job
→ spawn Worker → Worker 写 ocr_* 表 → Event → UI ocr.get_result
```

## 验收

- [x] database-schema.md 覆盖 MVP 全部表 + OCR + background_job
- [x] process-model.md 明确 UI 主进程禁止 OCR
- [x] ADR-0002 accepted
- [x] Storage / OCR / Runtime 领域 README 可导航
- [x] 子 Change CHG-023 / CHG-024 已创建并依赖本文档

## 实际结果

设计文档已落库；实施见 CHG-023（Worker + opendesk.db 基础）、CHG-024（OCR 管线）。

## 后续项

- [CHG-023 opendesk.db + Worker 骨架](chg-20260720-023-opendesk-db-worker-skeleton.md)
- [CHG-025 Tesseract 语言包下载](chg-20260720-025-ocr-tesseract-model-download.md)
- [CHG-024 OCR Worker 管线](chg-20260720-024-ocr-worker-pipeline.md)
