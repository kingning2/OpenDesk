# Python Runtime Domain

## 职责

Python 是由 Rust 托管的 AI Runtime，负责 LLM、RAG、Agent、OCR、Browser、Provider、Queue 和 Worker 能力。

## 非职责

- 不负责 React 或 Tauri；
- 不直接操作 SQLite 或持久化业务状态；
- 不绕过 Rust 向前端发送消息；
- 不自行定义与 `contracts/` 重复的跨端 DTO。

## 稳定边界

```text
Rust → gateway → queue → worker → AI capability/provider
```

Rust负责 Sidecar 生命周期、权限、业务状态、存储和前端事件转发。

## 当前状态

目前处于 Architecture Skeleton 阶段，Python package 和 Sidecar 仍以骨架为主。LLM 基础层已建立以下进程内边界：

```text
provider（适配与注册） → llm（Protocol 与标准错误） → model（中立类型）
```

- `model`、`llm`、`provider` 中的类型不是跨端 DTO；
- 上层能力依赖 `llm` Protocol，不直接依赖具体 Provider；
- `FakeChatModel` 仅用于测试和离线开发；
- `OpenAIChatModel` 使用官方 Python SDK 和 Chat Completions API，支持显式配置的 OpenAI 兼容端点；
- `GeminiChatModel` 使用官方 Google GenAI SDK 和原生 Generate Content API，默认模型为稳定版 `gemini-3.5-flash`，并支持 Python 进程内异步流式 Chunk；
- `.env.example` 当前将本地 Provider 路由预设为 `gemini`，并保留 OpenAI 配置作为可选 Adapter；
- Provider 不读取 `.env`，API Key 和模型配置必须由组合根显式注入；
- LLM 标准错误区分认证失败与权限拒绝，只暴露 HTTP 状态、Provider error code 和 Request ID 等安全诊断元数据；
- OpenAI/Gemini Adapter 尚未接入 Sidecar 组合根，Rust 密钥注入、Agent、RAG 和跨端调用仍须通过独立 Change Record 实施。

长期依赖选择见 [ADR-0001](../../decisions/python-runtime/adr-0001-python-llm-package-boundaries.md)。

## 日志边界

- Python 使用标准库 `logging`，经 stdout 输出 `runtime/log/entry/v1` JSON Lines；
- `shared.logging` 提供配置、上下文传播、脱敏和开发 payload 预览；
- Python 不直接写日志文件，Rust负责接管、展示、落盘和轮转；
- stderr 保留给未捕获异常和第三方原始输出；
- 日志只用于观测，不作为生命周期或健康控制协议。

## 直接相关领域

- [Contracts](../contracts/README.md)
- [Agent](../agent/README.md)
