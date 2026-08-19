# Python Sidecar Domain

## 职责

Python 是 **Rust 生态不够时的例外 Sidecar**，不是 AI Runtime。默认能力（含 LLM、Agent、存储、渠道）在 Rust 实现。

仅在 Change Record 写明「Rust 生态缺少可用实现」时才编写或扩展 `python/**`。已接受的例外示例：Playwright 浏览器会话（[ADR-0008](../../decisions/channel/adr-0008-browser-login-hybrid.md)）。

Sidecar 仍由 Rust 托管生命周期；请求经本机 HTTP，契约来自 `contracts/`。

## 非职责

- 不作为默认 AI / LLM / RAG / Agent 实现层；
- 不负责 React 或 Tauri；
- 不直接操作 SQLite 或持久化业务状态；
- 不绕过 Rust 向前端发送消息；
- 不自行定义与 `contracts/` 重复的跨端 DTO；
- 不在未论证生态缺口时新增 Python 包或 handler。

## 稳定边界

```text
默认：React → Rust（实现，含 AI）

例外：Rust → gateway →（仅该缺口能力）Python sidecar
```

Rust 负责 Sidecar 生命周期、权限、业务状态、存储和前端事件转发。

## 有效 ADR

- [ADR-0009-python-only-when-rust-insufficient](../../decisions/python-runtime/adr-0009-python-only-when-rust-insufficient.md)
- [ADR-0008-browser-login-hybrid](../../decisions/channel/adr-0008-browser-login-hybrid.md)（Playwright 为例外）

## 当前状态

Architecture Skeleton：sidecar 与 `agent_ping` 骨架仍在，不表示 AI 必须走 Python。后续能力默认落 Rust；Python 只补生态缺口。

## 日志边界

- Python 使用标准库 `logging`，经 stdout 输出 `runtime/log/entry/v1` JSON Lines；
- `shared.logging` 提供配置、上下文传播、脱敏和开发 payload 预览；
- Python 不直接写日志文件，Rust 负责接管、展示、落盘和轮转；
- stderr 保留给未捕获异常和第三方原始输出；
- 日志只用于观测，不作为生命周期或健康控制协议。

## 直接相关领域

- [Contracts](../contracts/README.md)
- [Agent](../agent/README.md)
- [Runtime](../runtime/README.md)
