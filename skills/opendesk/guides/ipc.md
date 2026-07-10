# IPC Guide

Tauri IPC 是 React 与 Rust 之间的**唯一**通信通道。

## 调用链

```
Feature UI  →  @desk/platform/ipc  →  src-tauri commands  →  UseCase
```

## 契约定义

```
contracts/schema/v1/<feature>/ipc/<command>.request.schema.json
contracts/schema/v1/<feature>/ipc/<command>.response.schema.json
```

## 命令命名

```
<feature>_<action>_<resource>

chat_list_threads
mail_sync_account
agent_run_task
```

## Rust 注册

```rust
// src-tauri — 骨架示意
.invoke_handler(tauri::generate_handler![
    chat_list_threads,
])
```

## 流式响应

长时 AI 任务：

1. IPC 返回 `task_id`（立即）
2. Rust 订阅 Python 流
3. Rust 通过 Tauri Event `emit("agent:token", payload)` 推送
4. React 经 `platform.listen` 订阅

禁止 React 轮询 Python 或 WebSocket 直连 sidecar。

## 错误传递

IPC 错误映射为 contracts 定义的 `ErrorDto`，含 `code`、`message`、`trace_id`。

## 工具

```bash
python skills/opendesk/scripts/create_ipc.py --feature chat --command list_threads
```

## 相关

- [frontend.md](frontend.md)
- [rust-python-ipc.md](rust-python-ipc.md) — Rust ↔ Python sidecar HTTP
- [../recipes/add-ipc.md](../recipes/add-ipc.md)
