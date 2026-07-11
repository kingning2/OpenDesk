---
id: CHG-20260711-004-rust-sidecar-log-ingest
title: Rust接管Python Sidecar日志
type: change
status: proposed
priority: P0
owner: developer-b
domain: python-runtime
parent: EPIC-20260711-001-python-sidecar-logging
depends_on:
  - CHG-20260711-003-python-structured-logging
blocks: []
milestone: python-runtime-foundation
created: 2026-07-11
updated: 2026-07-11
contracts: runtime/log/entry/v1
related: []
---

# Rust接管Python Sidecar日志

## 目标

由 Rust并发接管 Python Sidecar stdout/stderr，将 v1 JSON日志映射到 tracing，并安全处理非结构化输出。

## 非目标

- 本 Python 分支不实现该任务；
- 日志不驱动重启、健康判断或业务状态。

## 交接规范

- stdout：逐行解析 `runtime/log/entry/v1`；合法日志按 level 写入 tracing。
- stdout 非法 JSON：包装为 `source=python_stdout` 的非结构化 warning，不得使读取任务退出。
- stderr：逐行标记为 `source=python_stderr`，作为 warning/error 兜底。
- stdout/stderr 必须并发持续读取，避免任一管道填满造成 Sidecar 阻塞。
- Rust负责最终控制台、文件、过滤和轮转；Python 永不直接落盘。
- 生命周期依据退出码、health 和超时，不依据日志消息文本。
- 当前 `sync_contracts.py` 不支持嵌套 `attributes/exception` 的正确类型生成；Rust实现前须先决定并评审 codegen 增强，禁止生成错误的 `String` 占位类型后直接使用。

## 验收

- [ ] 合法 JSON日志映射到 tracing
- [ ] 非 JSON stdout 安全降级
- [ ] stderr 原始异常可见
- [ ] 两条管道并发读取且退出时正确收尾
- [ ] trace_id/task_id/feature 可供 Rust结构化检索

## 交接状态

CHG-20260711-003 已完成，日志 v1 Contract 和 Python输出已就绪。等待 Developer B 在 `role/rust` 分支审批并实施。
