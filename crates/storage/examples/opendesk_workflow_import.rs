//! Import email-agent workflow data into `opendesk.db`.
//!
//! Idempotent — every write is an `ON CONFLICT DO UPDATE` upsert, safe to re-run.
//!
//! 作者：coisini
//! 创建时间：2026-08-07
//!
//! Usage:
//!   cargo run -p storage --example opendesk_workflow_import -- [db-path] [email-agent-data-dir]

use std::env;
use std::path::{Path, PathBuf};

use diesel::prelude::*;
use ports::workflow::{
    WorkflowBindingRecord, WorkflowRuleRecord, WorkflowScriptRecord, WorkflowStageRecord,
    WorkflowStore, WorkflowTemplateRecord,
};
use serde_json::Value;
use storage::opendesk_db::schema::workflow_stage::dsl as stage;
use storage::opendesk_db::OpendeskDb;
use storage::workflow::SqliteWorkflowStore;

fn main() {
    let args: Vec<String> = env::args().collect();
    let db_path = args
        .get(1)
        .map(PathBuf::from)
        .or_else(default_db_path)
        .expect("usage: opendesk_workflow_import [db-path] [email-agent-data-dir]");
    let data_dir = args
        .get(2)
        .map(PathBuf::from)
        .unwrap_or_else(default_data_dir);

    let db = OpendeskDb::open(&db_path).expect("open opendesk.db");
    let store = SqliteWorkflowStore::new(db.clone());

    let templates_doc = read_json(&data_dir.join("workflow-templates.json"));
    let templates_updated = templates_doc
        .get("updated")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    for template in templates_doc
        .get("templates")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let id = str_at(template, "id");
        let canvas_name = if id == "_global" {
            "workflow-stages.json".to_string()
        } else {
            format!("workflow-stages-{id}.json")
        };
        let canvas = read_json(&data_dir.join(&canvas_name));
        let canvas_json = serde_json::to_string(&canvas).expect("serialize canvas");

        let template_record = WorkflowTemplateRecord {
            id: id.to_string(),
            name: str_at(template, "name").to_string(),
            template_type: str_at(template, "type").to_string(),
            canvas_json,
            canvas_version: canvas
                .get("version")
                .and_then(Value::as_i64)
                .map(|v| v.to_string()),
            canvas_updated: canvas
                .get("updated")
                .and_then(Value::as_str)
                .map(str::to_string),
            created_at: str_at(template, "createdAt").to_string(),
            updated_at: templates_updated.clone(),
        };
        store
            .upsert_template(template_record)
            .expect("upsert template");

        for stage in canvas
            .get("stages")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let stage_record = WorkflowStageRecord {
                template_id: id.to_string(),
                id: str_at(stage, "id").to_string(),
                name: str_at(stage, "name").to_string(),
                note: stage
                    .get("note")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                ord: stage.get("order").and_then(Value::as_i64).unwrap_or(0),
                ai_level: stage
                    .get("aiLevel")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                x: stage.get("x").and_then(Value::as_i64),
                y: stage.get("y").and_then(Value::as_i64),
                scripts_json: to_json_array(stage, "scripts"),
                script_conds_json: to_json_array(stage, "scriptConds"),
            };
            store.upsert_stage(stage_record).expect("upsert stage");
        }
    }

    for (account_id, template_id) in templates_doc
        .get("bindings")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
    {
        let binding = WorkflowBindingRecord {
            account_id: account_id.clone(),
            template_id: template_id.as_str().unwrap_or("").to_string(),
        };
        store.upsert_binding(binding).expect("upsert binding");
    }

    let rules_doc = read_json(&data_dir.join("flow-rules.json"));
    for rule in rules_doc
        .get("rules")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let rule_record = WorkflowRuleRecord {
            id: str_at(rule, "id").to_string(),
            name: str_at(rule, "name").to_string(),
            from_stages_json: to_json_array(rule, "from_stages"),
            to_stage: str_at(rule, "to_stage").to_string(),
            trigger_keywords_json: to_json_array(rule, "trigger_keywords"),
            trigger_tags_json: to_json_array(rule, "trigger_tags"),
            auto_reply: rule
                .get("auto_reply")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            auto_advance: rule
                .get("auto_advance")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            reply_script_id: rule
                .get("reply_script_id")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(str::to_string),
        };
        store.upsert_rule(rule_record).expect("upsert rule");
    }

    let scripts_doc = read_json(&data_dir.join("scripts.json"));
    let scripts_updated = scripts_doc
        .get("updated")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    for (idx, script) in scripts_doc
        .get("scripts")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let script_record = WorkflowScriptRecord {
            id: str_at(script, "id").to_string(),
            stage: script
                .get("stage")
                .and_then(Value::as_str)
                .map(str::to_string),
            category_l1: script
                .get("category1")
                .and_then(Value::as_str)
                .map(str::to_string),
            category_l2: script
                .get("category2")
                .and_then(Value::as_str)
                .map(str::to_string),
            trigger_text: script
                .get("trigger")
                .and_then(Value::as_str)
                .map(str::to_string),
            description: script
                .get("description")
                .and_then(Value::as_str)
                .map(str::to_string),
            from_stage: script
                .get("from_stage")
                .and_then(Value::as_str)
                .map(str::to_string),
            to_stage: script
                .get("to_stage")
                .and_then(Value::as_str)
                .map(str::to_string),
            tags_json: to_json_array(script, "tags"),
            content: str_at(script, "content").to_string(),
            needs_boss_input: false,
            boss_input_hint: None,
            sort_order: idx as i64,
            created_at: scripts_updated.clone(),
            updated_at: scripts_updated.clone(),
        };
        store.upsert_script(script_record).expect("upsert script");
    }

    let stage_count = db
        .with_conn(|conn| {
            stage::workflow_stage
                .count()
                .get_result::<i64>(conn)
                .map_err(|e| ports::repository::StoreError::Unavailable(e.to_string()))
        })
        .expect("count stages");
    println!(
        "imported into {}:\n  workflow_template = {}\n  workflow_binding = {}\n  workflow_stage  = {}\n  workflow_rule    = {}\n  workflow_script  = {}",
        db_path.display(),
        store.list_templates().expect("count templates").len(),
        store.list_bindings().expect("count bindings").len(),
        stage_count,
        store.list_rules().expect("count rules").len(),
        store.list_scripts(None, None, None).expect("count scripts").len(),
    );
}

fn str_at<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("")
}

fn to_json_array(value: &Value, key: &str) -> String {
    value
        .get(key)
        .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "[]".to_string()))
        .unwrap_or_else(|| "[]".to_string())
}

fn read_json(path: &Path) -> Value {
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn default_db_path() -> Option<PathBuf> {
    env::var("LOCALAPPDATA")
        .ok()
        .map(|root| PathBuf::from(root).join("OpenDesk").join("opendesk.db"))
}

fn default_data_dir() -> PathBuf {
    PathBuf::from("D:/Desktop/_Projects/email-agent/data")
}
