# Layer Architecture

## 三层模型

```
┌──────────────────────────────────────────────────────────┐
│ Layer 1: React（Presentation）                            │
│                                                          │
│  apps/desktop/src/          Feature UI 与路由             │
│  packages/ui/               纯展示组件                    │
│  packages/platform/         IPC 封装（唯一 Tauri 入口）    │
└────────────────────────────┬─────────────────────────────┘
                             │ invoke / listen (platform only)
┌────────────────────────────▼─────────────────────────────┐
│ Layer 2: Rust（Application Core）                         │
│                                                          │
│  apps/desktop/src-tauri/    Tauri commands · 事件转发      │
│  crates/app/                应用组装与启动                 │
│  crates/kernel/             event bus · task scheduler    │
│  crates/ports/              共享 Port trait               │
│  crates/<feature>/          Feature 业务（app + domain）   │
│  crates/storage/            SQLite 实现                   │
│  crates/runtime/            Python sidecar 生命周期        │
└────────────────────────────┬─────────────────────────────┘
                             │ 本机 IPC（contracts/openapi）
┌────────────────────────────▼─────────────────────────────┐
│ Layer 3: Python（AI Runtime）                             │
│                                                          │
│  python/sidecar/            进程入口 · 管理面 API          │
│  python/packages/gateway/   请求路由                      │
│  python/packages/worker/    异步任务                      │
│  python/packages/llm/       模型调用                      │
│  python/packages/rag/       检索增强                      │
└──────────────────────────────────────────────────────────┘
```

## 各层职责

### React Layer

| 负责 | 禁止 |
|------|------|
| UI 渲染、交互、主题、动画 | 业务规则、SQL、AI 逻辑 |
| 本地 UI 状态（表单、展开/折叠） | 直接 `invoke()`（Feature UI） |
| 通过 `@desk/platform/ipc` 调 Rust | `import @tauri-apps/api`（Feature UI） |

### Rust Layer

| 负责 | 禁止 |
|------|------|
| 业务编排、权限、缓存 | `unwrap()` / `panic!()` |
| SQLite 读写（经 storage） | Feature 间直接 `use` |
| Python sidecar 生命周期 | 阻塞 UI 线程 |
| 结构化日志（tracing） | Python 直连前端事件 |
| Tauri IPC 命令与事件转发 | |

### Python Layer

| 负责 | 禁止 |
|------|------|
| LLM / RAG / OCR / Embedding | GUI、Tauri、React |
| Agent / MCP / Browser 自动化 | SQLite、业务状态持久化 |
| Queue / Worker 异步执行 | 未评审的 HTTP Server |

## Rust 内部分层（六边形）

```
┌─────────────────────────────────────┐
│  app/（Application / UseCase）       │  ← 编排，无 IO
├─────────────────────────────────────┤
│  domain/（Entity / Value Object）    │  ← 纯领域，无框架依赖
├─────────────────────────────────────┤
│  ports/（trait 定义）                │  ← 接口
├─────────────────────────────────────┤
│  infra/（storage 实现，可选在 crate 内）│  ← SQL / HTTP / FS
└─────────────────────────────────────┘
```

## 层间通信矩阵

| From \ To | React | Rust | Python | SQLite |
|-----------|-------|------|--------|--------|
| React | ✅ | ✅ IPC | ❌ | ❌ |
| Rust | ✅ Events | ✅ | ✅ IPC | ✅ |
| Python | ❌ | ✅ | ✅ | ❌ |

## 相关文档

- [feature-boundary.md](feature-boundary.md)
- [../guides/ipc.md](../guides/ipc.md)
- [dependency.md](dependency.md)
