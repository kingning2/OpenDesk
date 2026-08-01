# Logging Guide

Rust 使用 `tracing`。在应用边界记录操作名、实体/任务 id、耗时和结果；底层库不要重复打印同一错误。

- 可关联字段：`trace_id`、`task_id`、`workflow_id`、`account_id`。
- 错误日志保留原因链，但面向 UI 的错误使用稳定 Contract。
- API Key、密码、令牌、邮件完整正文和个人数据不得进入普通日志。
- React 只记录可操作的 UI/IPC 诊断，不用 `console.log` 代替错误状态。

长任务开始、状态转换、完成与失败各记录一次，避免循环内噪声。
