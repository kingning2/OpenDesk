//! 导入文档用例：解析 → 分块 → 向量化 → 入库。

use std::sync::Arc;

use agent::embedding::Embedder;
use ports::knowledge::{KnowledgeDocumentRecord, KnowledgeStore};
use uuid::Uuid;

use crate::chunk::chunk_markdown;
use crate::parse::{parse_to_markdown, source_type};

/// 导入结果：向量化后的文档记录。
#[derive(Debug, Clone)]
pub struct ImportOutcome {
    pub document: KnowledgeDocumentRecord,
}

/// 把上传文件解析、向量化并写入知识库。
///
/// 流程：`parse_to_markdown` → `chunk_markdown` → 批量嵌入 → 逐块入库 → 置为 ready。
/// 嵌入失败时清理已建文档并返回错误，不残留空记录。
pub struct ImportDocument;

impl ImportDocument {
    /// 执行一次文档导入。
    ///
    /// # 参数
    /// - `name` — 原始文件名（含扩展名）
    /// - `bytes` — 文件原始字节
    /// - `store` — 知识库持久化端口
    /// - `embedder` — 本地嵌入服务（512 维）
    ///
    /// # Errors
    ///
    /// 解析失败、嵌入整体失败或落库失败时返回错误描述。
    pub async fn execute(
        name: &str,
        bytes: &[u8],
        store: Arc<dyn KnowledgeStore>,
        embedder: Arc<dyn Embedder>,
    ) -> Result<ImportOutcome, String> {
        let source_type = source_type(name).ok_or_else(|| format!("不支持的文件类型: {name}"))?;
        let markdown = parse_to_markdown(name, bytes)
            .await
            .map_err(|error| error.to_string())?;
        let chunks = chunk_markdown(&markdown);
        if chunks.is_empty() {
            return Err("文档解析后没有可检索的内容".to_string());
        }

        let id = Uuid::new_v4().to_string();
        let document = store
            .create_document(&id, name, source_type)
            .map_err(|error| error.to_string())?;

        // 嵌入是 CPU 密集（onnx），放到 blocking 线程池。当前 Embedder 对整批一次性
        // 推理，失败即整批失败：清理已建文档后返回错误，避免残留 'parsing' 空记录。
        let chunks_for_embed = chunks.clone();
        let embedder_for_embed = Arc::clone(&embedder);
        let embeddings = match tokio::task::spawn_blocking(move || {
            embedder_for_embed.embed_texts(&chunks_for_embed)
        })
        .await
        {
            Ok(Ok(embeddings)) => embeddings,
            Ok(Err(error)) => {
                let _ = store.delete_document(&id);
                return Err(error.to_string());
            }
            Err(error) => {
                let _ = store.delete_document(&id);
                return Err(error.to_string());
            }
        };
        if embeddings.len() != chunks.len() {
            let _ = store.delete_document(&id);
            return Err("嵌入结果与分块数量不一致，已中止导入".to_string());
        }

        let mut insert_failures = 0usize;
        for (index, (chunk, embedding)) in chunks.iter().zip(embeddings.iter()).enumerate() {
            if store
                .insert_chunk(&id, chunk, index as i64, embedding)
                .is_err()
            {
                insert_failures += 1;
            }
        }
        if insert_failures > 0 {
            let _ = store.delete_document(&id);
            return Err(format!("{insert_failures} 个分块写入失败，已中止导入"));
        }

        store
            .finish_document(&id, embeddings.len() as i64)
            .map_err(|error| error.to_string())?;

        tracing::info!(
            document_id = %id,
            name,
            chunks = chunks.len(),
            "knowledge document imported"
        );
        Ok(ImportOutcome { document })
    }
}
