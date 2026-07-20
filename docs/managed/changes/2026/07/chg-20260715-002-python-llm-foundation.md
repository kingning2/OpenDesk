---
id: CHG-20260715-002-python-llm-foundation
title: 建立 Python LLM 基础边界
type: change
status: completed
priority: P1
owner: codex
domain: python-runtime
parent: none
depends_on: []
blocks: []
milestone: ai-runtime-foundation
created: 2026-07-15
updated: 2026-07-15
contracts: none
related: [ADR-0001-python-llm-package-boundaries]
---

# 建立 Python LLM 基础边界

## 目标

在不接入真实模型和跨端 API 的前提下，建立 `model`、`llm`、`provider` 的最小可测试边界，使后续 Provider 适配不影响 Agent 上层代码。

## 非目标

- 不引入 LangChain 或具体厂商 SDK。
- 不实现 Agent、RAG、Tool Calling、流式传输或持久化。
- 不修改 Contract、Rust、React、Sidecar 和 Gateway。

## 背景

三个 Python package 当前均为空骨架，目标架构文档对它们的职责存在重叠。本次只建立中立模型类型、LLM Port、Provider Registry 和 Fake Provider，为后续真实适配器提供稳定边界。

## 影响与边界

- 修改范围：`python/packages/model/**`、`python/packages/llm/**`、`python/packages/provider/**`、本 Change Record、对应 ADR 和 Python Runtime Domain。
- 不修改范围：其他 Python 能力包、业务层、跨端实现。
- Contract：无；本次类型仅供 Python 进程内部使用。
- 跨层：否。
- 跨 Feature：否。
- 风险：过早扩大公共 API；通过最小字段、Protocol 和 Fake 实现控制。

## 依赖关系

- 父任务：无。
- 前置任务：无。
- 阻塞任务：无。

## 实施方案

1. 在 `model` 定义不可变的对话请求、响应、消息、用量和能力类型。
2. 在 `llm` 定义异步 Chat Model Protocol 和标准错误。
3. 在 `provider` 实现 Provider Registry 与确定性的 Fake Chat Model。
4. 明确依赖方向为 `provider -> llm -> model`，上层仅依赖 `llm` 与 `model`。
5. 使用标准库单元测试覆盖类型校验、Protocol、注册表和 Fake 调用。

## 验收

- [x] 静态检查通过
- [x] 相关测试通过
- [x] 架构边界检查通过
- [x] 实际结果已回填

## 实际结果

- `model` 新增不可变的 Chat 消息、请求、响应、Token 用量和能力类型，并进行最小运行时校验。
- `llm` 新增异步 `ChatModel` Protocol 与稳定错误码。
- `provider` 新增 `ProviderRegistry` 和无外部 I/O 的 `FakeChatModel`。
- `llm` 仅依赖 `model`，`provider` 依赖 `llm` 与 `model`，依赖关系写入各 package 的 `pyproject.toml`。
- 新增 11 个标准库单元测试，覆盖值校验、Protocol、注册、解析、重复/未知 Provider 和 Fake 响应。
- 验证通过：隔离 Python 3.14（满足项目 `>=3.13`）运行 11 个测试；Ruff check；Ruff format check；`check_boundary.py`；`check_imports.py`；`git diff --check`。
- 计划偏差：仓库本地 `.venv` 指向已不可用的 Python 3.10，因此验证使用 uv 隔离解释器；未改变实现范围。

## 后续项

- 在独立 Change 中评审并实现 LangChain 或厂商 Provider Adapter。
- 后续可单独修复本地 `.venv`，使 `pnpm lint:python` 包装命令恢复直接可用。
