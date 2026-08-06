//! 列出知识库文档用例。

use std::sync::Arc;

use ports::knowledge::{KnowledgeDocumentRecord, KnowledgeStore};

/// 列出所有知识库文档（最新更新在前）。
pub struct ListDocuments;

impl ListDocuments {
    /// 执行列表查询。
    ///
    /// # 参数
    /// - `store` — 知识库持久化端口
    ///
    /// # Errors
    ///
    /// 存储查询失败时返回错误描述。
    pub fn execute(store: Arc<dyn KnowledgeStore>) -> Result<Vec<KnowledgeDocumentRecord>, String> {
        store.list_documents().map_err(|error| error.to_string())
    }
}
