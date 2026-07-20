---
id: CHG-20260716-001-provider-permission-error
title: 区分 Provider 权限错误并保留安全诊断元数据
type: change
status: completed
priority: P1
owner: codex
domain: python-runtime
parent: none
depends_on: [CHG-20260715-004-openai-chat-provider]
blocks: []
milestone: ai-runtime-provider
created: 2026-07-16
updated: 2026-07-16
contracts: none
related: [ADR-0001-python-llm-package-boundaries]
---

# 区分 Provider 权限错误并保留安全诊断元数据

## 目标

将 Provider 的认证失败与权限拒绝明确区分，并在标准错误中保留脱敏后的 HTTP 状态、Provider 错误码和 Request ID，使模型权限问题可以直接定位。

## 非目标

- 不暴露 API Key、Prompt、原始响应正文或 Provider 原始错误消息。
- 不修改 Contract、Rust、React、Sidecar、Agent 或流式调用。
- 不改变 OpenAI 请求参数和响应模型。

## 背景

真实测试发现 OpenAI `403 PermissionDeniedError` 被归一化为 `ProviderAuthenticationError`，导致 `model_not_found` 和 Request ID 丢失，难以区分无效密钥与 Project 模型权限不足。

## 影响与边界

- 修改范围：`python/packages/llm/**`、`python/packages/provider/**`、本 Change Record 和 Python Runtime Domain。
- 不修改范围：跨端层、环境变量加载和业务能力包。
- Contract：无；错误类型仅供 Python 进程内部使用。
- 跨层：否。
- 跨 Feature：否。
- 风险：诊断字段携带敏感正文；只提取状态码、Provider error code 和 Request ID，错误消息继续使用固定文本。

## 依赖关系

- 父任务：无。
- 前置任务：CHG-20260715-004-openai-chat-provider。
- 阻塞任务：无。

## 实施方案

1. 为 LLM 标准错误增加不可变的安全 Provider 元数据。
2. 新增权限拒绝错误码，OpenAI 401 映射认证失败，403 映射权限拒绝。
3. 从 SDK 状态错误中提取安全元数据，不复制原始响应消息和正文。
4. 增加权限映射、元数据和脱敏回归测试。

## 验收

- [x] 静态检查通过
- [x] 相关测试通过
- [x] 架构边界检查通过
- [x] 实际结果已回填

## 实际结果

- `llm` 新增不可变的 `ProviderErrorMetadata`，只接受合法 HTTP 状态和受限字符集的 Provider error code、Request ID。
- `LLMError` 保持固定脱敏消息，并在字符串表示中附加安全元数据，使测试 traceback 可以直接定位状态码、错误码和请求 ID。
- 新增 `ProviderPermissionDeniedError`（`provider_permission_denied`）；OpenAI 401 继续映射认证失败，403 改为权限拒绝。
- OpenAI 状态错误统一提取安全元数据；服务端原始 message/body、API Key 和 Prompt 不进入标准错误或异常显示链，不符合标识符格式的元数据会被丢弃。
- 合计 27 个测试：26 个通过，1 个真实测试默认跳过；新增测试覆盖权限映射、安全元数据渲染、非法元数据拒绝和敏感正文脱敏。
- 使用当前 Project 重跑真实测试，按预期仍因模型权限返回 403，但 traceback 已准确显示 `ProviderPermissionDeniedError`、`model_not_found` 和 Request ID，不再误报认证失败。
- 验证通过：Ruff check；Ruff format check（43 个文件）；`check_boundary.py`；`check_imports.py`；公开类型导入冒烟检查；暂存区和工作区 `git diff --check`。
- 计划偏差：无。未修改 Contract、Rust 或前端。

## 后续项

- 无。
