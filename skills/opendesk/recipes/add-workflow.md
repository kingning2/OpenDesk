# Recipe: Add Workflow

1. 用 Contract 定义启动、状态、取消/恢复与事件 payload。
2. 用 `WorkflowDefinition` 表达 DAG，不为单步调用创建工作流。
3. 为真实副作用实现 `NodeExecutor`，并注册到 `ExecutorRegistry`。
4. 设置重试、run policy 与幂等边界。
5. 持久化检查点后再发布状态事件，确保可恢复。
6. 由 `crates/app` 暴露 Tauri IPC，React 只消费状态。

验证状态转换、失败重试、取消和恢复。参考 `crates/workflow_runtime`。
