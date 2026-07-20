---
id: CHG-20260715-004-openai-chat-provider
title: 实现真实 OpenAI Chat Provider
type: change
status: completed
priority: P1
owner: codex
domain: python-runtime
parent: none
depends_on: [CHG-20260715-002-python-llm-foundation]
blocks: []
milestone: ai-runtime-provider
created: 2026-07-15
updated: 2026-07-15
contracts: none
related: [ADR-0001-python-llm-package-boundaries]
---

# 实现真实 OpenAI Chat Provider

## 目标

使用官方 `openai` Python SDK 实现首个真实 `ChatModel`，支持 OpenAI 及符合官方 Chat Completions 规范的兼容端点，并保持现有 Provider 抽象不变。

## 非目标

- 不引入 LangChain 或 LangGraph。
- 不实现 Agent、流式输出、Tool Calling、RAG 或持久化。
- 不修改 Contract、Rust、React、Sidecar 或 Gateway。
- 不实现 Rust 向 Python Sidecar 的密钥注入。

## 背景

CHG-20260715-002 已建立 `provider -> llm -> model` 的进程内边界和 Fake Provider。本次在该边界内增加官方 OpenAI SDK 适配器，验证真实 Provider 可以在不污染上层 Protocol 的情况下接入。

## 影响与边界

- 修改范围：`python/packages/provider/**`、`python/packages/llm/**`、本 Change Record 和 Python Runtime Domain。
- 不修改范围：其他 Python 能力包、跨端层、`.env` 加载和 Rust 进程配置注入。
- Contract：无；仅使用 Python 进程内类型。
- 跨层：否。
- 跨 Feature：否。
- 风险：第三方响应或异常泄露密钥、Prompt、原始正文；通过固定错误消息、禁止 Adapter 请求日志和密钥 `repr` 控制。

## 依赖关系

- 父任务：无。
- 前置任务：CHG-20260715-002-python-llm-foundation。
- 阻塞任务：无。

## 实施方案

1. 为 Provider 包增加受限版本的官方 `openai` SDK 依赖。
2. 增加显式注入且隐藏密钥表示的 `OpenAIChatConfig`。
3. 使用 `AsyncOpenAI.chat.completions.create()` 实现非流式 `OpenAIChatModel`，标准化参数和响应。
4. 扩展 LLM 标准错误并对 SDK 异常做脱敏映射。
5. 使用 Mock 覆盖适配行为；增加默认跳过、显式启用的真实端点测试。

## 验收

- [x] 静态检查通过
- [x] 相关测试通过
- [x] 架构边界检查通过
- [x] 实际结果已回填

## 实际结果

- `provider` 增加 `openai>=2.38,<3`，并新增显式配置的 `OpenAIChatConfig`；API Key 不出现在配置 `repr` 中。
- `OpenAIChatModel` 使用 `AsyncOpenAI.chat.completions.create()` 实现非流式 Chat，支持三种消息角色、请求模型覆盖、`temperature` 和 `max_completion_tokens` 参数映射。
- 响应被标准化为进程内 `ChatResponse`，包含文本、实际模型、finish reason 和输入/输出 Token 用量；空或畸形响应转换为稳定错误。
- `llm` 新增配置、认证和响应错误码；认证、限流、超时、连接、400 与 5xx SDK 异常均使用固定脱敏文本映射，不记录请求内容，也不保留可能暴露原始正文的异常显示链。
- 新增 Mock 离线测试和显式开启的真实端点测试。合计 23 个测试：22 个通过，1 个真实测试按预期跳过；原有 11 个测试全部通过。
- 验证通过：Python 3.14 隔离环境运行测试；Ruff check；Ruff format check（43 个文件）；`check_boundary.py`；`check_imports.py`；新增公开类型导入冒烟检查；暂存区和工作区 `git diff --check`。
- 计划偏差：无。未读取项目 `.env`，未调用真实模型端点，未修改 Contract、Rust 或前端。

## 后续项

- Rust 启动 Python Sidecar 时的 Provider 配置和密钥注入另建跨层 Change。
- LangChain/LangGraph 在 Agent 编排阶段按实际需求另行评审。
