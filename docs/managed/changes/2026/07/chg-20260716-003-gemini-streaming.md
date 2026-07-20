---
id: CHG-20260716-003-gemini-streaming
title: 实现 Gemini 完整流式输出
type: change
status: completed
priority: P1
owner: codex
domain: python-runtime
parent: none
depends_on: [CHG-20260716-002-gemini-chat-provider]
blocks: []
milestone: ai-runtime-provider
created: 2026-07-16
updated: 2026-07-16
contracts: none
related: [ADR-0001-python-llm-package-boundaries]
---

# 实现 Gemini 完整流式输出

## 目标

为 Python 进程内 LLM 边界增加中立流式 Chunk 和 Streaming Protocol，并让 Gemini Adapter 使用官方异步流式 API 逐块输出完整文本及最终诊断信息。

## 非目标

- 不为 OpenAI Adapter 增加流式实现。
- 不修改 Contract、Rust、React、Sidecar 或跨端事件协议。
- 不实现 Tool Calling、Agent、多模态输入或流式持久化。

## 背景

现有 Gemini Live 测试只验证非空响应，不显示模型正文。用户需要真正的逐块实时输出，而不是生成完成后一次性打印。

## 影响与边界

- 修改范围：`python/packages/model/**`、`python/packages/llm/**`、`python/packages/provider/**`、本 Change Record 和 Python Runtime Domain。
- 不修改范围：跨端层和其他 Provider 的能力声明。
- Contract：无；流式类型仍是 Python 进程内模型，不是跨端 DTO。
- 跨层：否。
- 跨 Feature：否。
- 风险：流式增量、最终 finish reason 和 usage 可能分布在不同 Chunk；使用可选元数据 Chunk 并在 Live 测试中聚合验证。

## 依赖关系

- 父任务：无。
- 前置任务：CHG-20260716-002-gemini-chat-provider。
- 阻塞任务：无。

## 实施方案

1. 在 `model` 增加不可变的 `ChatStreamChunk`。
2. 在 `llm` 增加继承 `ChatModel` 的 `StreamingChatModel` Protocol。
3. 在 Gemini Adapter 使用 `client.aio.models.generate_content_stream()`，逐块标准化文本、模型、finish reason 和 usage。
4. Live 测试实时打印每个文本增量，结束后打印完整元数据并保留断言。
5. 增加离线流式内容、能力、空流和异常映射测试。

## 验收

- [x] 静态检查通过
- [x] 相关测试通过
- [x] 架构边界检查通过
- [x] 实际结果已回填

## 实际结果

- `model` 新增不可变 `ChatStreamChunk`，允许文本增量或仅包含结束元数据的最终 Chunk。
- `llm` 新增 `StreamingChatModel` Protocol；Gemini 同时声明 `CHAT` 与 `STREAMING` 能力。
- Gemini Adapter 使用官方 `client.aio.models.generate_content_stream()`，逐块标准化正文、模型、finish reason 和 token usage，并复用安全错误映射。
- Live 测试改为即时打印每个文本增量，结束后打印 provider、model、chunk 数、finish reason 和 usage；移除测试输出 token 上限，避免思考 token 挤占正文造成截断。
- 离线验证：model 7 项、llm 5 项、provider 35 项全部通过，其中 2 项 Live 按预期开关跳过。
- 真实验证：`gemini-3.5-flash` 输出 5 个 Chunk，`finish_reason=stop`，测试通过。
- Ruff check、Ruff format check、`check_boundary.py`、`check_imports.py`、`git diff --check` 和 staged diff check 通过。
- 聚合 `check_architecture.py` 仍报告两个既存 Rust `build.rs` 的 `unwrap/expect/panic`，本 Change 未修改 Rust，且本次相关边界检查独立通过。
- 与计划偏差：无功能范围偏差；根据真实验证结果移除了 Live 测试的 256 token 限制。

## 后续项

- 跨端流式传输和 OpenAI 流式 Adapter 分别通过后续 Change 实施。
