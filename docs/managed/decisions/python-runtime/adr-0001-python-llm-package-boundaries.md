---
id: ADR-0001-python-llm-package-boundaries
title: Python LLM 包依赖边界
status: accepted
domain: python-runtime
created: 2026-07-15
supersedes: none
---

# Python LLM 包依赖边界

## Context

`model`、`llm`、`provider` 已存在为空 package，但目标文档尚未明确接口、内部类型和具体 Provider 实现的依赖方向。若 Agent 直接依赖 LangChain 或厂商 SDK，替换 Provider 会影响上层能力。

## Decision

- `model` 保存 Python Runtime 内部的中立、不可变模型类型，不作为跨端 Contract。
- `llm` 定义上层使用的 Chat Model Protocol 和标准错误，只依赖 `model`。
- `provider` 实现 Protocol、注册表与具体适配器，依赖 `llm` 和 `model`。
- Sidecar 组合根在后续任务中选择并注入 Provider；Agent 不直接构造具体 Provider。
- LangChain 如被采用，只能位于具体适配器或 Agent 编排实现内部，其类型不得进入跨端 Contract。

## Alternatives

- 让 `llm` 同时包含厂商实现：接口和基础设施耦合，未采用。
- 让 Agent 直接使用 LangChain类型：框架类型会扩散到业务和契约，未采用。
- 本次直接接入真实 Provider：无法先独立验证边界，留给后续 Change。

## Consequences

- 正面影响：上层可使用 Fake 测试，具体 Provider 可替换。
- 成本与限制：增加一层适配，并需维护内部类型转换。
- 后续兼容要求：跨端字段仍须先在 `contracts/` 定义；内部类型不得冒充 codegen DTO。
