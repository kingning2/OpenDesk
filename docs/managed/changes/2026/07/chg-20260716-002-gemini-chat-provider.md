---
id: CHG-20260716-002-gemini-chat-provider
title: 实现 Gemini Chat Provider
type: change
status: completed
priority: P1
owner: codex
domain: python-runtime
parent: none
depends_on: [CHG-20260716-001-provider-permission-error]
blocks: []
milestone: ai-runtime-provider
created: 2026-07-16
updated: 2026-07-16
contracts: none
related: [ADR-0001-python-llm-package-boundaries]
---

# 实现 Gemini Chat Provider

## 目标

使用 Google 官方 `google-genai` Python SDK 实现原生 Gemini `ChatModel`，将本地默认 Provider 预设为 `gemini`、默认模型预设为稳定版 `gemini-3.5-flash`。

## 非目标

- 不使用 Gemini 的 OpenAI 兼容端点或 LangChain。
- 不实现 Agent、流式输出、Tool Calling、多模态输入、Vertex AI 或持久化。
- 不修改 Contract、Rust、React、Sidecar 或 Gateway。
- 不在 Provider 包中读取 `.env`。

## 背景

OpenAI Adapter 已验证 Provider 边界，但当前 OpenAI Project 没有目标模型权限。Google 官方文档确认 `gemini-3.5-flash` 为稳定模型，推荐使用生产级 `google-genai` SDK，并支持异步 `generate_content`。

## 影响与边界

- 修改范围：`.env.example`、`python/packages/provider/**`、本 Change Record 和 Python Runtime Domain。
- 不修改范围：实际 `.env`、跨端层和其他 AI 能力包。
- Contract：无；继续使用 Python 进程内 `ChatModel`、`ChatRequest`、`ChatResponse`。
- 跨层：否。
- 跨 Feature：否。
- 风险：Gemini 消息角色、响应用量和错误结构与 OpenAI 不同；通过原生类型映射、固定脱敏消息和离线 Mock 测试控制。

## 依赖关系

- 父任务：无。
- 前置任务：CHG-20260716-001-provider-permission-error。
- 阻塞任务：无。

## 实施方案

1. 增加受限版本的官方 `google-genai` SDK 依赖。
2. 新增隐藏 API Key 的 `GeminiChatConfig`，模型默认值为 `gemini-3.5-flash`。
3. 使用 `client.aio.models.generate_content()` 实现非流式文本 Chat，映射角色、系统指令、生成参数、响应文本、模型、finish reason 和 Token 用量。
4. 将 Google SDK 与 HTTP 异常映射到现有 LLM 标准错误，并保留安全诊断元数据。
5. 更新 `.env.example` 的 Provider 路由和 Gemini 官方变量，增加离线 Mock 与显式 Live 测试。

## 验收

- [x] 静态检查通过
- [x] 相关测试通过
- [x] 架构边界检查通过
- [x] 实际结果已回填

## 实际结果

- 根据 Google 官方模型、SDK 和 Generate Content 文档确认 `gemini-3.5-flash` 是稳定模型 ID，官方 Python SDK 为 `google-genai`，API Key 官方变量为 `GEMINI_API_KEY`。
- `provider` 增加 `google-genai>=2.10,<3` 和直接使用的 `httpx` 依赖；隔离验证环境解析到 `google-genai 2.12.0`。
- 新增 `GeminiChatConfig`，隐藏 API Key，默认模型为 `gemini-3.5-flash`，并显式配置 60 秒超时和 2 次重试。
- 新增原生 `GeminiChatModel`，使用异步 `client.aio.models.generate_content()`；System 消息映射为 `system_instruction`，User/Assistant 映射为 `user/model`，并传递模型覆盖、temperature 和 max output tokens。
- Gemini 响应被标准化为文本、模型版本、finish reason 和包含思考 Token 的用量；Google API/HTTP 异常映射到现有 LLM 标准错误并保留安全元数据。
- `.env.example` 默认路由更新为 `OPENDESK_LLM_PROVIDER=gemini`、`OPENDESK_LLM_MODEL=gemini-3.5-flash`，新增 `GEMINI_API_KEY`，保留 OpenAI 配置。
- OpenAI 与 Gemini Live 测试同时检查全局 Live 开关和所选 Provider，避免开启一次测试时请求两个厂商。
- 合计 41 个测试：39 个通过，Gemini/OpenAI 两个真实测试按预期跳过；覆盖配置、角色、参数、响应、用量、错误、脱敏和 Registry。
- 验证通过：Ruff check；Ruff format check（46 个文件）；`check_boundary.py`；`check_imports.py`；Gemini/OpenAI 公开导入冒烟检查；暂存区和工作区 `git diff --check`。
- 计划偏差：无。未读取或修改实际 `.env`，未调用 Gemini 真实端点，未修改 Contract、Rust 或前端。

## 后续项

- Rust 向 Sidecar 注入所选 Provider 和密钥仍由后续跨层 Change 处理。
