# Recipe: Add Agent

## 修改顺序

1. Contract: agent IPC + event schema
2. `crates/agent/` UseCase 骨架
3. `python/packages/agent/` 包骨架
4. `runtime` 注册 sidecar 路由（骨架）
5. 流式：Rust Event 转发占位

## 禁止

- Python Agent 直连 React
- Agent 状态写 SQLite（由 Rust 持久化）

## 模板

[../examples/python/agent_handler.py](../examples/python/agent_handler.py)
