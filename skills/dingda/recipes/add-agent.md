# Recipe: Add Agent

Agent / LLM **默认在 Rust 实现**。不要默认增加 Python handler。

## 修改顺序

1. Contract: agent IPC + event schema
2. `apps/desktop/src-tauri/src/agent.rs` — UseCase（默认含模型调用）
3. 仅当 Change Record 论证 Rust 生态不够：才加 `python/packages/gateway/handlers/` 与 runtime sidecar 路由
4. 流式：Rust Event 转发占位

## 禁止

- 把新的 LLM / Agent 能力默认放到 Python
- Python Agent 直连 React
- Agent 状态写 SQLite（由 Rust 持久化）

## 模板

[../examples/python/agent_handler.py](../examples/python/agent_handler.py)（仅 sidecar 例外时使用）
