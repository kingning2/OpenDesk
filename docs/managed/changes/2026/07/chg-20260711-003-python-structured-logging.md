---
id: CHG-20260711-003-python-structured-logging
title: Python结构化日志Contract与实现
type: change
status: completed
priority: P0
owner: developer-c
domain: python-runtime
parent: EPIC-20260711-001-python-sidecar-logging
depends_on: []
blocks:
  - CHG-20260711-004-rust-sidecar-log-ingest
milestone: python-runtime-foundation
created: 2026-07-11
updated: 2026-07-11
contracts: runtime/log/entry/v1
related: []
---

# Python结构化日志Contract与实现

## 目标

定义 v1 日志契约，并让 Python Sidecar 和共享运行时通过 stdout 输出一行一个 JSON日志事件。

## 非目标

- 不修改 `crates/**`；
- 不直接写日志文件；
- 不实现日志UI、上传、轮转或生命周期控制。

## 背景

Python 当前只有零散的标准库 logging 调用，Sidecar 入口仍使用 `print()`，Rust尚无稳定可解析的输出协议。

## 影响与边界

- 修改范围：`contracts/`、`python/packages/shared`、`python/sidecar`、相关 Python 测试和 Python Runtime Domain；
- 不修改范围：Rust、React、业务 Feature；
- Contract：新增非破坏性 v1 Schema并更新 Changelog；
- 跨层：Python stdout → Rust；
- 跨 Feature：无；
- 风险：开发日志可能包含业务文本，必须截断并脱敏。

## 依赖关系

- 父任务：EPIC-20260711-001-python-sidecar-logging；
- 前置任务：无；
- 阻塞任务：CHG-20260711-004-rust-sidecar-log-ingest。

## 实施方案

1. 新增日志 JSON Schema，先固定跨进程字段。
2. 在 shared 包实现 Formatter、上下文、脱敏和 payload preview。
3. Sidecar 启动时配置 logging 并移除 `print()`。
4. 增加标准库 unittest，验证格式、上下文、隐私和异常降级。

## 验收

- [x] 每条日志是一行合法 JSON并符合 v1 Schema
- [x] 日志级别和运行环境由安全默认值控制
- [x] 上下文字段同步/异步隔离
- [x] Prompt/回复预览截断并脱敏
- [x] 异常堆栈脱敏、路径清理并限制长度
- [x] Sidecar 不再使用 `print()`
- [x] Python lint、测试、Contract和架构检查通过

## 实际结果

已新增 `runtime/log/entry/v1` Schema和 Contract Changelog 条目；在 `shared.logging` 实现 JSON Lines Formatter、ContextVar 上下文、隐私清洗、payload preview 和安全降级；Sidecar 与 Gateway 已接入共享日志能力。未修改 Rust。

验证结果：在 Python 3.14（满足项目 `>=3.13`）完成真实 stdout JSON烟测；Ruff check/format 通过，8 个 unittest 通过，Contract、boundary、imports 检查通过，`git diff --check` 通过，实施范围内无直接 `print()`。

已知限制：现有 `sync_contracts.py` 不支持嵌套 object 的正确类型生成，且会同时修改 Rust/TypeScript。本次遵守分支边界未运行跨端 codegen；Rust正式消费该 Schema 前应先增强 codegen 或使用严格的手写解析边界，并另行评审。

## 后续项

- Rust接管见 CHG-20260711-004-rust-sidecar-log-ingest，已解除前置依赖阻塞并等待 Developer B 实施。
