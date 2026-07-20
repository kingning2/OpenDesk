---
id: ADR-0002-heavy-work-worker-process
title: 重任务必须在独立 Rust Worker 进程执行
status: accepted
domain: runtime
created: 2026-07-20
supersedes: none
---

# 重任务必须在独立 Rust Worker 进程执行

## Context

OpenDesk 是 Tauri 桌面应用：主进程同时承载 WebView 与 Rust IPC。OCR、PDF 渲染、批量文件处理等任务 CPU/内存占用高、耗时长。

若在主进程执行：

- UI 线程卡顿、输入延迟；
- 单进程崩溃导致整个桌面退出；
- 难以限制并发与资源配额。

用户明确要求：**Rust 处理时不能影响 UI 进程，必须单独开进程。**

## Decision

### 1. 三进程模型

- **Tauri 主进程**：UI + 轻量 Rust Core（短 SQL、入队、事件）。
- **`opendesk-worker` 独立二进制**：OCR 及所有重 CPU/IO 任务。
- **Python Sidecar**：仅 AI 推理（既有架构）。

### 2. 任务协调

- 统一使用 `opendesk.db.background_job` 队列表。
- 主进程：**只** `INSERT queued` + spawn/monitor Worker。
- Worker：**claim → 执行 → 写结果表 → 更新 status**。
- UI 通过 Tauri Event + 轮询 job 状态获取进度，**禁止** Worker 直接调用 WebView API。

### 3. OCR 首版即遵守本 ADR

OCR 不得在主进程或 Python Sidecar 内做图像识别循环；**Tesseract** 识别引擎跑在 Worker（见 [ADR-0003](../ocr/adr-0003-tesseract-local-model-on-demand-download.md)）。

Python 可接收 **已 OCR 的文本** 做 AI 后处理（可选），文本由主进程从 DB 读取后经 Contract 传入。

### 4. 数据库并发

- `opendesk.db` 使用 **WAL** 模式，支持主进程读 + Worker 写。
- 长事务仅在 Worker 内；主进程事务须短（<50ms 目标）。

### 5. 现有爬虫

YouTube 爬虫当前 in-process；**不强制本 ADR retrofit**。新增强 CPU 任务一律 Worker；爬虫迁移 Worker 为独立 Change。

## Alternatives

| 方案 | 未选原因 |
|------|----------|
| 主进程 tokio spawn 阻塞线程池 | 仍共享进程内存与崩溃域；不符合用户要求 |
| OCR 放 Python Sidecar | 与「Rust 处理」方向不符；且 Sidecar 已有 AI 职责 |
| 每任务 spawn 一次性进程 | 启动开销大；采用常驻 Worker + 队列 |
| 系统 Job Object / 线程优先级 | 不能隔离崩溃；仅缓解卡顿 |

## Consequences

**正面：**

- UI 响应性可保证；
- Worker 可独立重启、限并发；
- 与 Sidecar 模式一致，团队易理解。

**成本：**

- 新增 `opendesk-worker` crate/binary、部署与升级路径；
- 需设计 job 队列、取消、进度协议；
- 双进程打开 SQLite 须 WAL + 迁移规范。

**兼容要求：**

- OCR 及后续重任务 Change 须引用本 ADR；
- `check_architecture.py` 后续可增加规则：OCR 引擎 crate 不得被 `crates/app` 直接依赖。
