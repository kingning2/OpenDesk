# Recipe: Add Agent Capability

Agent 能力在 Rust 内实现：

1. 定义输入、输出、事件与错误 Contract。
2. 在调用 Feature 中组装业务上下文与只读 Query Port。
3. 使用 `agent::llm::LlmClient` 调模型；模型协议与 Skill 基建留在 `crates/agent`，业务 Prompt/用例留在 Feature。
4. 多步骤任务接入 `crates/workflow_runtime`，为外部副作用定义 Executor。
5. 通过 Tauri IPC 返回结果或 run id，通过 Event 推送进度。
6. 发送邮件、改客户状态等高权限动作仍需明确的人机确认边界。

密钥只从安全存储读取，不写入 Contract 或日志。
