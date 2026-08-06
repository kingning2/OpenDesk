# Logging Guide

Rust 使用 `tracing`。**每个后端操作都必须有日志**：在应用边界记录操作名、实体/任务 id、开始与结果；底层库不要重复打印同一错误。

## 必记日志点

每个后端操作（Tauri command / UseCase / worker handler / 后台任务）至少两条：

1. **开始**：`tracing::info!(操作名, 实体/id, "… started")`。
2. **结果**：成功 `info!`，失败 `warn!/error!`（保留原因链，含 `error.to_string()`）。

长任务（下载、OCR、导入、同步）额外记：

3. **阶段/进度**：低频记录（如每 10% 或每 N MB），`tracing::debug!`，禁止每个 chunk/循环迭代打一次。
4. **终态**：完成/失败各一次，失败原因可关联 UI 的稳定 Contract 错误码。

## 字段约定

- 可关联字段：`trace_id`、`task_id`、`workflow_id`、`account_id`、`job_id`。
- 错误日志保留原因链，但面向 UI 的错误使用稳定 Contract。
- API Key、密码、令牌、邮件完整正文和个人数据不得进入普通日志。
- React 只记录可操作的 UI/IPC 诊断，不用 `console.log` 代替错误状态。

## 反例

- 只记录结果不记录开始（无从定位耗时与调用来源）。
- 循环内每条记录打日志（刷屏、掩盖异常）。
- 底层库打印上层已记录的错误（重复）。
- `unwrap`/`expect` 路径上无日志（违反「所有操作有日志」）。

