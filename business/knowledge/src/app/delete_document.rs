//! 删除知识库文档用例。

use std::sync::Arc;

use ports::knowledge::KnowledgeStore;

/// 删除一个知识库文档（级联删除其分块与向量）。
pub struct DeleteDocument;

impl DeleteDocument {
    /// 执行文档删除。
    ///
    /// # 参数
    /// - `store` — 知识库持久化端口
    /// - `document_id` — 目标文档 id
    ///
    /// # 返回值
    /// 文档存在并被删除时返回 `true`；不存在返回 `false`。
    ///
    /// # Errors
    ///
    /// 存储失败时返回错误描述。
    pub fn execute(store: Arc<dyn KnowledgeStore>, document_id: &str) -> Result<bool, String> {
        match store.delete_document(document_id) {
            Ok(()) => Ok(true),
            Err(ports::repository::StoreError::NotFound) => Ok(false),
            Err(error) => Err(error.to_string()),
        }
    }
}
