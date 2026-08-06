//! 知识库：把客户上传的文档解析为 Markdown，分块并向量化入库，供聊天 RAG 检索。
//!
//! 解析/分块在 `parse` / `chunk` 模块；业务用例在 `app`（import / list / delete）。
//! 向量化复用 `agent::embedding::Embedder`，持久化通过 `ports::KnowledgeStore`。

pub mod app;
pub mod chunk;
pub mod ocr;
pub mod parse;
pub mod tools;

pub use app::{DeleteDocument, ImportDocument, ImportOutcome, ListDocuments};
pub use chunk::chunk_markdown;
pub use parse::parse_to_markdown;
pub use tools::{detect_tool, download_tool, DownloadError, DownloadProgress, ToolId, ToolStatus};
