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

目前处于 Architecture Skeleton 阶段，Python package 和 Sidecar 以骨架为主。具体建设计划必须通过独立 Change Record 管理。

## 日志边界

- Python 使用标准库 `logging`，经 stdout 输出 `runtime/log/entry/v1` JSON Lines；
- `shared.logging` 提供配置、上下文传播、脱敏和开发 payload 预览；
- Python 不直接写日志文件，Rust负责接管、展示、落盘和轮转；
- stderr 保留给未捕获异常和第三方原始输出；
- 日志只用于观测，不作为生命周期或健康控制协议。

## 直接相关领域

- [Contracts](../contracts/README.md)
- [Agent](../agent/README.md)
