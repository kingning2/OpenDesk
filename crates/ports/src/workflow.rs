//! Workflow persistence port — script snippets.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-21

use crate::repository::StoreError;

/// A single script snippet record.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
#[derive(Debug, Clone)]
pub struct ScriptSnippetRecord {
    pub id: String,
    pub source_id: String,
    pub title: String,
    pub stage: Option<String>,
    pub trigger_text: Option<String>,
    pub description: Option<String>,
    pub from_stage: Option<String>,
    pub to_stage: Option<String>,
    /// JSON array of tag strings, stored as text.
    pub tags_json: String,
    pub body_text: String,
    pub category_l1: Option<String>,
    pub category_l2: Option<String>,
    pub needs_boss_input: bool,
    pub boss_input_hint: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Input to create or update a script snippet.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
#[derive(Debug, Clone)]
pub struct ScriptSnippetWriteInput {
    /// `None` → insert; `Some` → upsert by id.
    pub id: Option<String>,
    pub title: String,
    pub stage: Option<String>,
    pub trigger_text: Option<String>,
    pub description: Option<String>,
    pub from_stage: Option<String>,
    pub to_stage: Option<String>,
    /// JSON array of tag strings.
    pub tags_json: String,
    pub body_text: String,
    pub category_l1: Option<String>,
    pub category_l2: Option<String>,
    pub needs_boss_input: bool,
    pub boss_input_hint: Option<String>,
    pub sort_order: i64,
}

/// Workflow / script-snippet storage contract.
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
pub trait ScriptSnippetStore: Send + Sync {
    /// List snippets, optionally filtered by category_l1 / category_l2 / free-text search.
    ///
    /// # 参数
    /// - `category_l1` — optional L1 filter (e.g. `"KOL"`)
    /// - `category_l2` — optional L2 filter (e.g. `"阶段一"`)
    /// - `query`       — optional substring match against title + trigger_text + body_text
    fn list(
        &self,
        category_l1: Option<&str>,
        category_l2: Option<&str>,
        query: Option<&str>,
    ) -> Result<Vec<ScriptSnippetRecord>, StoreError>;

    /// Get one snippet by id.
    fn get(&self, id: &str) -> Result<ScriptSnippetRecord, StoreError>;

    /// Create or update a snippet.
    fn save(&self, input: ScriptSnippetWriteInput) -> Result<ScriptSnippetRecord, StoreError>;

    /// Delete a snippet by id.
    fn delete(&self, id: &str) -> Result<(), StoreError>;
}
