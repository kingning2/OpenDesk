# Rust Guide

适用范围：`crates/**`、`apps/desktop/src-tauri/**`

## 职责

Rust 是**唯一业务核心**与**唯一协调者**。

| 负责 | 禁止 |
|------|------|
| Application · Storage · Sidecar 生命周期 | `unwrap()` · `expect()` · `panic!()` |
| Event Bus · Task Scheduler | Feature 间直接 `use` |
| Permission · Cache · Logging | 阻塞 UI 线程 |
| Tauri IPC 与事件转发 | 无限循环线程 |

## Crate 结构

```
crates/<feature>/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── app/       # UseCase — 无 IO
    └── domain/    # 纯领域类型
```

## 错误处理

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChatError {
    #[error("thread not found: {0}")]
    ThreadNotFound(String),
    // ...
}

pub type ChatResult<T> = Result<T, ChatError>;
```

## 日志

```rust
use tracing::{info, instrument};

#[instrument(skip(repo), fields(thread_id))]
pub fn get_thread(repo: &dyn ThreadRepo, thread_id: &str) -> ChatResult<Thread> {
    info!("fetching thread");
    // ...
}
```

## Tauri 命令（骨架）

```rust
#[tauri::command]
pub async fn chat_threads_list(
    state: State<'_, AppState>,
) -> Result<Vec<ThreadDto>, String> {
    // delegate to UseCase — skeleton returns empty
    Ok(vec![])
}
```

## Workspace 注册

新 crate 必须加入根 `Cargo.toml` `[workspace.members]`。

## Lint

```bash
pnpm lint:rust
# 或 cargo lint
```

## 相关

- [error.md](error.md)
- [logging.md](logging.md)
- [../architecture/layers.md](../architecture/layers.md)
