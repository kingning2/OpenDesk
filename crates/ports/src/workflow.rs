//! Workflow persistence port — email-agent workflow templates, stages, rules, scripts.
//!
//! 作者：coisini
//! 创建时间：2026-08-07

use crate::repository::StoreError;

/// A workflow template row (`workflow_template`).
///
/// 作者：coisini
/// 创建时间：2026-08-07
#[derive(Debug, Clone)]
pub struct WorkflowTemplateRecord {
    pub id: String,
    pub name: String,
    pub template_type: String,
    /// The whole `workflow-stages{_<id>}.json` canvas verbatim.
    pub canvas_json: String,
    pub canvas_version: Option<String>,
    pub canvas_updated: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// An account → template binding (`workflow_binding`).
///
/// 作者：coisini
/// 创建时间：2026-08-07
#[derive(Debug, Clone)]
pub struct WorkflowBindingRecord {
    pub account_id: String,
    pub template_id: String,
}

/// A workflow canvas stage (`workflow_stage`, composite key template_id + id).
///
/// 作者：coisini
/// 创建时间：2026-08-07
#[derive(Debug, Clone)]
pub struct WorkflowStageRecord {
    pub template_id: String,
    pub id: String,
    pub name: String,
    pub note: Option<String>,
    pub ord: i64,
    pub ai_level: Option<String>,
    pub x: Option<i64>,
    pub y: Option<i64>,
    /// JSON array of script ids.
    pub scripts_json: String,
    /// JSON array of script condition strings.
    pub script_conds_json: String,
}

/// A routing rule (`workflow_rule`).
///
/// 作者：coisini
/// 创建时间：2026-08-07
#[derive(Debug, Clone)]
pub struct WorkflowRuleRecord {
    pub id: String,
    pub name: String,
    /// JSON array of from-stage ids.
    pub from_stages_json: String,
    pub to_stage: String,
    /// JSON array of trigger keywords.
    pub trigger_keywords_json: String,
    /// JSON array of trigger tags.
    pub trigger_tags_json: String,
    pub auto_reply: bool,
    pub auto_advance: bool,
    pub reply_script_id: Option<String>,
}

/// A script-snippet record (`workflow_script`).
///
/// 作者：coisini
/// 创建时间：2026-08-07
#[derive(Debug, Clone)]
pub struct WorkflowScriptRecord {
    pub id: String,
    pub stage: Option<String>,
    pub category_l1: Option<String>,
    pub category_l2: Option<String>,
    pub trigger_text: Option<String>,
    pub description: Option<String>,
    pub from_stage: Option<String>,
    pub to_stage: Option<String>,
    /// JSON array of tag strings.
    pub tags_json: String,
    pub content: String,
    pub needs_boss_input: bool,
    pub boss_input_hint: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Workflow definition storage contract.
///
/// 作者：coisini
/// 创建时间：2026-08-07
pub trait WorkflowStore: Send + Sync {
    /// List all workflow templates, ordered by created_at.
    fn list_templates(&self) -> Result<Vec<WorkflowTemplateRecord>, StoreError>;

    /// Get one template by id (includes full canvas_json).
    fn get_template(&self, id: &str) -> Result<Option<WorkflowTemplateRecord>, StoreError>;

    /// List all account → template bindings.
    fn list_bindings(&self) -> Result<Vec<WorkflowBindingRecord>, StoreError>;

    /// List all routing rules.
    fn list_rules(&self) -> Result<Vec<WorkflowRuleRecord>, StoreError>;

    /// List scripts, optionally filtered by category_l1 / category_l2 / free-text search.
    ///
    /// # 参数
    /// - `category_l1` — optional L1 filter (e.g. `"KOL"`)
    /// - `category_l2` — optional L2 filter (e.g. `"阶段一"`)
    /// - `query`       — optional substring match against trigger_text + description + content
    fn list_scripts(
        &self,
        category_l1: Option<&str>,
        category_l2: Option<&str>,
        query: Option<&str>,
    ) -> Result<Vec<WorkflowScriptRecord>, StoreError>;

    /// Idempotent upsert by primary key.
    fn upsert_template(&self, record: WorkflowTemplateRecord) -> Result<(), StoreError>;

    /// Idempotent upsert by primary key (`account_id`).
    fn upsert_binding(&self, record: WorkflowBindingRecord) -> Result<(), StoreError>;

    /// Idempotent upsert by composite key (`template_id`, `id`).
    fn upsert_stage(&self, record: WorkflowStageRecord) -> Result<(), StoreError>;

    /// Idempotent upsert by primary key.
    fn upsert_rule(&self, record: WorkflowRuleRecord) -> Result<(), StoreError>;

    /// Idempotent upsert by primary key.
    fn upsert_script(&self, record: WorkflowScriptRecord) -> Result<(), StoreError>;
}
