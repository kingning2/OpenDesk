# Runtime / Worker Domain

## 职责

Rust 是唯一运行时。Tauri 主进程负责 IPC、轻量业务与任务调度；`opendesk-worker` 处理可独立轮询的后台任务；`crates/workflow_runtime` 提供应用内多步骤工作流。

```text
React → Tauri IPC → Rust main
                    ├─ async 业务 / SQLite
                    ├─ workflow_runtime
                    └─ background_job → opendesk-worker
```

## 当前事实

- `crates/worker` 已实现独立二进制和 `background_job` 轮询。
- Worker 已注册 `imap_sync` 与 `crawler_email_enrich` handler，并启动已启用账户的 IMAP IDLE watcher。
- Tauri 主进程也提供周期 IMAP 同步调度；手动同步当前在 Rust 内执行并更新 sync cursor。
- `crates/workflow_runtime` 已包含 DAG、状态机、执行器注册、检查点和恢复能力。
- OCR handler/完整模型管线仍以 OCR Domain 与对应 Change 为准。

## 边界

- CPU 密集和批量任务不得阻塞 Tauri 主线程。
- Worker 不操作 WebView；UI 只经 IPC/Event 与持久化状态获知进度。
- 共享 SQLite 访问使用短事务、幂等任务与明确恢复策略。
