//! 长对话自动摘要：会话消息超过阈值且新增足够多时，把对话要点生成摘要并写入长期记忆。
//!
//! 由 `chat_send` 在流式返回后异步执行，不阻塞本次发送。摘要写入 `chat_memory`
//!（kind='digest'），后续请求可被向量检索到，跨会话回忆；同时更新
//! `chat_session.summary_state_json.last_digest_seq` 作为增量书签。

use std::sync::Arc;

use agent::embedding::Embedder;
use agent::llm::LlmClient;
use agent::prompt::Prompt;
use ports::chat::{ChatMemoryStore, ChatStore};
use serde_json::{json, Value};

/// 触发摘要的最低消息总数（条）。
const DIGEST_MIN_MESSAGES: usize = 20;
/// 距上次摘要后需新增至少多少条消息才再次摘要。
const DIGEST_NEW_MIN: i64 = 8;

/// 读取上次摘要书签（`{"last_digest_seq": N}`），缺省为 0。
fn last_digest_seq(chat_store: &dyn ChatStore, session_id: &str) -> i64 {
    chat_store
        .get_summary_state(session_id)
        .ok()
        .flatten()
        .and_then(|json| serde_json::from_str::<Value>(&json).ok())
        .and_then(|value| value.get("last_digest_seq").and_then(Value::as_i64))
        .unwrap_or(0)
}

/// 达到阈值时生成并写入一次会话摘要；未达阈值则直接返回。
///
/// # 参数
/// - `client` — LLM 客户端（生成摘要）
/// - `chat_store` — 会话/消息持久化端口
/// - `memory_store` — 长期记忆端口
/// - `embedder` — 本地嵌入服务（摘要向量化）
/// - `session_id` — 目标会话
///
/// # Errors
///
/// 加载消息、调用 LLM、嵌入或落库失败时返回错误（由调用方仅记日志）。
pub async fn maybe_digest(
    client: &LlmClient,
    chat_store: &dyn ChatStore,
    memory_store: &dyn ChatMemoryStore,
    embedder: Arc<dyn Embedder>,
    session_id: String,
) -> Result<(), String> {
    let records = chat_store
        .load_messages(&session_id)
        .map_err(|error| error.to_string())?;
    if records.len() < DIGEST_MIN_MESSAGES {
        return Ok(());
    }
    let last_seq = records.last().map(|record| record.seq).unwrap_or(0);
    let last_digest_seq = last_digest_seq(chat_store, &session_id);
    if last_seq - last_digest_seq < DIGEST_NEW_MIN {
        return Ok(());
    }

    // 只把上次摘要之后的新消息交给 LLM 总结。
    let recent = records
        .iter()
        .filter(|record| record.seq > last_digest_seq)
        .map(|record| format!("[{}] {}", record.role, record.content))
        .collect::<Vec<_>>()
        .join("\n");
    let prompt = format!(
        "请把下面这段对话提炼成「对话要点摘要」：涵盖出现的客户/公司名、重要事实、\
         数字、偏好与关键结论，用简洁的中文条目列出，不要省略具体数字与名字。\n\n对话：\n{recent}"
    );
    let digest = client
        .complete(&Prompt::new(
            "你是一个信息整理助手，只输出摘要内容本身。",
            &prompt,
        ))
        .await
        .map_err(|error| error.to_string())?
        .trim()
        .to_string();
    if digest.is_empty() {
        return Ok(());
    }

    let embedder = Arc::clone(&embedder);
    let digest_for_embed = digest.clone();
    let embedding = tokio::task::spawn_blocking(move || embedder.embed_text(&digest_for_embed))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?;

    memory_store
        .insert_memory("digest", &digest, Some(&session_id), &embedding)
        .map_err(|error| error.to_string())?;
    chat_store
        .set_summary_state(
            &session_id,
            &json!({ "last_digest_seq": last_seq }).to_string(),
        )
        .map_err(|error| error.to_string())?;

    tracing::info!(
        %session_id,
        messages = records.len(),
        new_messages = recent.lines().count(),
        digest_chars = digest.chars().count(),
        "chat memory digest written"
    );
    Ok(())
}
