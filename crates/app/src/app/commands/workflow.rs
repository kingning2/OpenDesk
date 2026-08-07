//! Workflow Tauri IPC commands — email-agent workflow templates / bindings / rules / scripts.
//!
//! 作者：coisini
//! 创建时间：2026-08-07

use std::collections::HashMap;

use common::contracts::{
    WorkflowIpcBindingListResponse, WorkflowIpcRuleListResponse, WorkflowIpcScriptListRequest,
    WorkflowIpcScriptListResponse, WorkflowIpcTemplateGetRequest, WorkflowIpcTemplateGetResponse,
    WorkflowIpcTemplateListResponse,
};

use crate::app::state::AppState;

/// List workflow templates with per-template binding counts.
///
/// # 返回值
/// Serialised template list (name + type + binding_count) and total.
#[tauri::command]
pub async fn workflow_template_list(
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowIpcTemplateListResponse, String> {
    let store = state.workflow_store.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let templates = store.list_templates().map_err(|e| e.to_string())?;
        let bindings = store.list_bindings().map_err(|e| e.to_string())?;

        let mut counts: HashMap<String, i64> = HashMap::new();
        for binding in &bindings {
            *counts.entry(binding.template_id.clone()).or_insert(0) += 1;
        }

        let total = templates.len() as i64;
        let templates_json = serde_json::to_string(
            &templates
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "id": t.id,
                        "name": t.name,
                        "template_type": t.template_type,
                        "canvas_json": t.canvas_json,
                        "canvas_version": t.canvas_version,
                        "canvas_updated": t.canvas_updated,
                        "binding_count": counts.get(&t.id).copied().unwrap_or(0),
                        "created_at": t.created_at,
                        "updated_at": t.updated_at,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .map_err(|e| e.to_string())?;

        Ok(WorkflowIpcTemplateListResponse {
            templates_json,
            total,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Fetch one workflow template (includes full canvas_json).
///
/// # 参数
/// - `request.id` — template id
///
/// # 返回值
/// Serialised single template.
#[tauri::command]
pub async fn workflow_template_get(
    state: tauri::State<'_, AppState>,
    request: WorkflowIpcTemplateGetRequest,
) -> Result<WorkflowIpcTemplateGetResponse, String> {
    let store = state.workflow_store.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let template = store
            .get_template(&request.id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("workflow template not found: {}", request.id))?;

        let template_json = serde_json::to_string(&serde_json::json!({
            "id": template.id,
            "name": template.name,
            "template_type": template.template_type,
            "canvas_json": template.canvas_json,
            "canvas_version": template.canvas_version,
            "canvas_updated": template.canvas_updated,
            "created_at": template.created_at,
            "updated_at": template.updated_at,
        }))
        .map_err(|e| e.to_string())?;

        Ok(WorkflowIpcTemplateGetResponse { template_json })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// List all workflow account → template bindings.
///
/// # 返回值
/// Serialised binding list and total.
#[tauri::command]
pub async fn workflow_binding_list(
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowIpcBindingListResponse, String> {
    let store = state.workflow_store.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let bindings = store.list_bindings().map_err(|e| e.to_string())?;

        let total = bindings.len() as i64;
        let bindings_json = serde_json::to_string(
            &bindings
                .iter()
                .map(|b| {
                    serde_json::json!({
                        "account_id": b.account_id,
                        "template_id": b.template_id,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .map_err(|e| e.to_string())?;

        Ok(WorkflowIpcBindingListResponse {
            bindings_json,
            total,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// List all workflow routing rules.
///
/// # 返回值
/// Serialised rule list and total.
#[tauri::command]
pub async fn workflow_rule_list(
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowIpcRuleListResponse, String> {
    let store = state.workflow_store.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let rules = store.list_rules().map_err(|e| e.to_string())?;

        let total = rules.len() as i64;
        let rules_json = serde_json::to_string(
            &rules
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.id,
                        "name": r.name,
                        "from_stages_json": r.from_stages_json,
                        "to_stage": r.to_stage,
                        "trigger_keywords_json": r.trigger_keywords_json,
                        "trigger_tags_json": r.trigger_tags_json,
                        "auto_reply": r.auto_reply,
                        "auto_advance": r.auto_advance,
                        "reply_script_id": r.reply_script_id,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .map_err(|e| e.to_string())?;

        Ok(WorkflowIpcRuleListResponse { rules_json, total })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// List workflow scripts with optional category / free-text filters.
///
/// # 参数
/// - `request` — optional category_l1, category_l2, query filters
///
/// # 返回值
/// Serialised script list with total count.
#[tauri::command]
pub async fn workflow_script_list(
    state: tauri::State<'_, AppState>,
    request: WorkflowIpcScriptListRequest,
) -> Result<WorkflowIpcScriptListResponse, String> {
    let store = state.workflow_store.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let scripts = store
            .list_scripts(
                request.category_l1.as_deref(),
                request.category_l2.as_deref(),
                request.query.as_deref(),
            )
            .map_err(|e| e.to_string())?;

        let total = scripts.len() as i64;
        let scripts_json = serde_json::to_string(
            &scripts
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.id,
                        "stage": r.stage,
                        "category_l1": r.category_l1,
                        "category_l2": r.category_l2,
                        "trigger_text": r.trigger_text,
                        "description": r.description,
                        "from_stage": r.from_stage,
                        "to_stage": r.to_stage,
                        "tags_json": r.tags_json,
                        "content": r.content,
                        "needs_boss_input": r.needs_boss_input,
                        "boss_input_hint": r.boss_input_hint,
                        "sort_order": r.sort_order,
                        "created_at": r.created_at,
                        "updated_at": r.updated_at,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .map_err(|e| e.to_string())?;

        Ok(WorkflowIpcScriptListResponse {
            scripts_json,
            total,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}
