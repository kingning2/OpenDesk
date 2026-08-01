# OpenDesk Architecture

```text
React（展示） → Tauri IPC → Rust（唯一协调者与运行时）
```

Rust 负责业务、SQLite、SMTP/IMAP、后台任务、Workflow Runtime、系统能力与 AI；AI 基建位于 `crates/agent`（`llm/`、`prompt/`、`skills/`）。

## 不变量

- React 不直连数据库、文件或模型服务。
- Feature 间只通过 Contract、Event 或 Query Port。
- 跨端顺序为 `Contract → codegen → Rust → React`。
- 重任务不阻塞 Tauri 主线程；Worker 不操作 WebView。

## 文档

- [`product-architecture.md`](product-architecture.md)：产品与端到端能力
- [`process-model.md`](process-model.md)：Tauri 主进程、Worker 与 Workflow Runtime
- [`database-schema.md`](database-schema.md)：SQLite 设计
- [`../managed/`](../managed/)：路线图、Domain、Change 与 ADR
- [`../../.cursor/rules/master.md`](../../.cursor/rules/master.md)：执行规则
