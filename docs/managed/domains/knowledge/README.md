# Knowledge Domain

## 职责

本地知识库的文档导入与向量检索：

- 文件解析为 Markdown（PDF / Word .docx / HTML / 纯文本 / Markdown）——**在 `opendesk-worker` 进程执行**，前端 `tauri-plugin-dialog` 选文件拿路径 → 主进程入队 `background_job` → worker 按路径读文件解析
- 解析工具链管理：Pandoc（docx/html）、Tesseract（PDF 扫描页 OCR）、PDFium（pdf2md 文本提取 / 页面渲染），前端触发下载、后端落盘 + 进度事件；工具缺失时回退纯 Rust（docx-rs / pdf-extract）
- Markdown 分块（按标题 + 语义单元边界）
- 复用本地嵌入服务（bge-small-zh-v1.5，512 维）向量化入库
- `knowledge.db`（rusqlite + sqlite-vec）存储文档、分块与向量
- 供聊天自动检索注入（RAG），并受 `knowledge_enabled` 开关控制

## 非职责

- 扫描版 PDF 的 OCR（OCR 领域，CHG-024 设计）
- LLM 主动检索工具（`search_knowledge` MCP 工具，未实现）
- 文档版本化 / 增量向量化（v1 上传即全量建块）

## 稳定边界

```text
apps/desktop knowledge page
  → @desk/platform (dialog pick + ipc/knowledge)
  → crates/app commands::knowledge (enqueue background_job)
  → opendesk-worker handlers::knowledge_import
  → crates/knowledge UseCase (parse → chunk → embed)
  → ports::KnowledgeStore trait
  → storage::knowledge::SqliteKnowledgeStore (knowledge.db)

crates/chat send_chat
  → ports::KnowledgeStore (检索注入)
  → agent::embedding::Embedder (查询向量化)
```

## 入口

| 类型 | 路径 |
|------|------|
| Crate | `crates/knowledge/`（解析 + UseCase） |
| Worker | `crates/worker/src/handlers/knowledge_import.rs` |
| Port | `crates/ports/src/knowledge.rs`、`crates/ports/src/background_job.rs` |
| Storage | `crates/storage/src/knowledge/`（knowledge.db） |
| IPC 命令 | `crates/app/src/app/commands/knowledge.rs` |
| 前端 | `apps/desktop/src/features/knowledge/` |
| Change | [CHG-20260806-001-knowledge-base-rag](../changes/2026/08/CHG-20260806-001-knowledge-base-rag.md)、[CHG-20260806-002-parser-tools](../changes/2026/08/CHG-20260806-002-parser-tools.md)、[CHG-20260806-003-knowledge-import-worker](../changes/2026/08/CHG-20260806-003-knowledge-import-worker.md) |

## 库文件

| 文件 | 内容 |
|------|------|
| `{data}/OpenDesk/knowledge.db` | `knowledge_doc` / `knowledge_chunk` / `knowledge_chunk_vec` |
| `{data}/OpenDesk/tools/` | Pandoc / Tesseract / PDFium 可执行文件 |

## 当前状态

- 规划 / 实施中（CHG-20260806-001 / 002 / 003）。

## 当前约束

- 向量维度固定 512（与 bge-small-zh-v1.5、chat_memory_vec 一致）。
- 向量检索依赖 sqlite-vec `vec0` 虚拟表，连接需先注册自动扩展。
- 文件选择由前端 `tauri-plugin-dialog` 完成，文件路径经 IPC + `background_job` 传 worker；主进程不再解析。
- worker 是独立进程，无法直接发 Tauri 事件；导入完成通知由主进程轮询 `knowledge_doc` 变化推送。
