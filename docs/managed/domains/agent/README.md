# Agent Domain

## 职责

Agent 领域负责 AI 规划、模型交互、结构化工具请求和任务执行状态，不负责业务持久化或绕过 Rust 执行高权限操作。

## 目标边界

- Python 产生结构化 `ToolCall`；
- Rust负责审批、授权和工具执行；
- Python 接收 `ToolResult` 后恢复任务；
- 长任务使用 `task_id`，支持暂停、恢复、取消和超时。

## 当前状态

目前只有包和示例 Handler 骨架。新的 Agent 内核设计应先创建 Change Record；形成长期技术选择时再建立 ADR。
