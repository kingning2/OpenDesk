---
id: EPIC-20260711-001-python-sidecar-logging
title: Python Sidecar 结构化日志
type: epic
status: in_progress
priority: P0
owner: developer-c
domain: python-runtime
milestone: python-runtime-foundation
created: 2026-07-11
updated: 2026-07-11
related: []
---

# Python Sidecar 结构化日志

## 目标

建立 Python 经 stdout 输出 JSON Lines、由 Rust接管并汇入 tracing 的跨进程日志基础。

## 非目标

- 本 Epic 不实现 Rust Sidecar 进程管理、文件落盘或日志轮转；
- 不把日志作为健康检查或自动重启协议；
- 不引入第三方 Python 日志依赖。

## 总体边界

- 主领域：Python Runtime；
- 涉及领域：Contracts、Rust Runtime；
- Contract：新增 v1 Runtime Log Entry Schema；
- 跨层：Python stdout → Rust，由正式 Contract 约束；
- 长期决策：沿用仓库既有 Rust接管 Sidecar stdout/stderr 规则。

## 子任务

| Child Change | 状态 | 负责人 | Depends on | 验收结果 |
|---|---|---|---|---|
| [Python日志Contract与实现](chg-20260711-003-python-structured-logging.md) | completed | Developer C | — | 8项测试及全部检查通过 |
| [Rust日志接管交接](chg-20260711-004-rust-sidecar-log-ingest.md) | proposed | Developer B | CHG-20260711-003 | Python协议已就绪 |

## 总体验收

- [x] Python stdout 每行输出合法 v1 JSON日志
- [x] Python日志测试和架构检查通过
- [ ] Rust接管实现由 Developer B 完成
- [ ] Python Runtime Domain 已更新

## 结果摘要

Python 子任务已完成；Epic 保持进行中，等待 Rust交接子任务。本分支未越权修改 Rust。
