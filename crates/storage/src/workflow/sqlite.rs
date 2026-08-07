//! Diesel-backed workflow store.
//!
//! 作者：coisini
//! 创建时间：2026-08-07

use diesel::prelude::*;
use ports::repository::StoreError;
use ports::workflow::{
    WorkflowBindingRecord, WorkflowRuleRecord, WorkflowScriptRecord, WorkflowStageRecord,
    WorkflowStore, WorkflowTemplateRecord,
};

use crate::opendesk_db::schema::workflow_binding::dsl as binding;
use crate::opendesk_db::schema::workflow_rule::dsl as rule;
use crate::opendesk_db::schema::workflow_script::dsl as script;
use crate::opendesk_db::schema::workflow_stage::dsl as stage;
use crate::opendesk_db::schema::workflow_template::dsl as templ;
use crate::opendesk_db::{
    NewWorkflowBindingRow, NewWorkflowRuleRow, NewWorkflowScriptRow, NewWorkflowStageRow,
    NewWorkflowTemplateRow, OpendeskDb, WorkflowBindingRow, WorkflowRuleRow, WorkflowScriptRow,
    WorkflowTemplateRow,
};

/// SQLite implementation of [`WorkflowStore`].
///
/// 作者：coisini
/// 创建时间：2026-08-07
pub struct SqliteWorkflowStore {
    db: OpendeskDb,
}

impl SqliteWorkflowStore {
    /// Wrap an existing [`OpendeskDb`] handle.
    ///
    /// 作者：coisini
    /// 创建时间：2026-08-07
    pub fn new(db: OpendeskDb) -> Self {
        Self { db }
    }
}

fn map_err(e: diesel::result::Error) -> StoreError {
    match e {
        diesel::result::Error::NotFound => StoreError::NotFound,
        other => StoreError::Unavailable(other.to_string()),
    }
}

fn template_row_to_record(row: WorkflowTemplateRow) -> WorkflowTemplateRecord {
    WorkflowTemplateRecord {
        id: row.id,
        name: row.name,
        template_type: row.template_type,
        canvas_json: row.canvas_json,
        canvas_version: row.canvas_version,
        canvas_updated: row.canvas_updated,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn binding_row_to_record(row: WorkflowBindingRow) -> WorkflowBindingRecord {
    WorkflowBindingRecord {
        account_id: row.account_id,
        template_id: row.template_id,
    }
}

fn rule_row_to_record(row: WorkflowRuleRow) -> WorkflowRuleRecord {
    WorkflowRuleRecord {
        id: row.id,
        name: row.name,
        from_stages_json: row.from_stages_json,
        to_stage: row.to_stage,
        trigger_keywords_json: row.trigger_keywords_json,
        trigger_tags_json: row.trigger_tags_json,
        auto_reply: row.auto_reply,
        auto_advance: row.auto_advance,
        reply_script_id: row.reply_script_id,
    }
}

fn script_row_to_record(row: WorkflowScriptRow) -> WorkflowScriptRecord {
    WorkflowScriptRecord {
        id: row.id,
        stage: row.stage,
        category_l1: row.category_l1,
        category_l2: row.category_l2,
        trigger_text: row.trigger_text,
        description: row.description,
        from_stage: row.from_stage,
        to_stage: row.to_stage,
        tags_json: row.tags_json,
        content: row.content,
        needs_boss_input: row.needs_boss_input,
        boss_input_hint: row.boss_input_hint,
        sort_order: row.sort_order,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

impl WorkflowStore for SqliteWorkflowStore {
    fn list_templates(&self) -> Result<Vec<WorkflowTemplateRecord>, StoreError> {
        self.db.with_conn(|conn| {
            templ::workflow_template
                .order(templ::created_at.asc())
                .select(WorkflowTemplateRow::as_select())
                .load::<WorkflowTemplateRow>(conn)
                .map(|rows| rows.into_iter().map(template_row_to_record).collect())
                .map_err(map_err)
        })
    }

    fn get_template(&self, id: &str) -> Result<Option<WorkflowTemplateRecord>, StoreError> {
        self.db.with_conn(|conn| {
            let row = templ::workflow_template
                .filter(templ::id.eq(id))
                .select(WorkflowTemplateRow::as_select())
                .first::<WorkflowTemplateRow>(conn)
                .optional()
                .map_err(map_err)?;
            Ok(row.map(template_row_to_record))
        })
    }

    fn list_bindings(&self) -> Result<Vec<WorkflowBindingRecord>, StoreError> {
        self.db.with_conn(|conn| {
            binding::workflow_binding
                .select(WorkflowBindingRow::as_select())
                .load::<WorkflowBindingRow>(conn)
                .map(|rows| rows.into_iter().map(binding_row_to_record).collect())
                .map_err(map_err)
        })
    }

    fn list_rules(&self) -> Result<Vec<WorkflowRuleRecord>, StoreError> {
        self.db.with_conn(|conn| {
            rule::workflow_rule
                .order(rule::id.asc())
                .select(WorkflowRuleRow::as_select())
                .load::<WorkflowRuleRow>(conn)
                .map(|rows| rows.into_iter().map(rule_row_to_record).collect())
                .map_err(map_err)
        })
    }

    fn list_scripts(
        &self,
        category_l1: Option<&str>,
        category_l2: Option<&str>,
        query: Option<&str>,
    ) -> Result<Vec<WorkflowScriptRecord>, StoreError> {
        self.db.with_conn(|conn| {
            let mut q = script::workflow_script
                .order((
                    script::category_l1.asc(),
                    script::category_l2.asc(),
                    script::sort_order.asc(),
                ))
                .into_boxed();

            if let Some(l1) = category_l1 {
                q = q.filter(script::category_l1.eq(l1));
            }
            if let Some(l2) = category_l2 {
                q = q.filter(script::category_l2.eq(l2));
            }
            if let Some(kw) = query {
                let pattern = format!("%{kw}%");
                q = q.filter(
                    script::trigger_text
                        .like(pattern.clone())
                        .or(script::description.like(pattern.clone()))
                        .or(script::content.like(pattern)),
                );
            }

            q.select(WorkflowScriptRow::as_select())
                .load::<WorkflowScriptRow>(conn)
                .map(|rows| rows.into_iter().map(script_row_to_record).collect())
                .map_err(map_err)
        })
    }

    fn upsert_template(&self, record: WorkflowTemplateRecord) -> Result<(), StoreError> {
        let row = NewWorkflowTemplateRow {
            id: record.id,
            name: record.name,
            template_type: record.template_type,
            canvas_json: record.canvas_json,
            canvas_version: record.canvas_version,
            canvas_updated: record.canvas_updated,
            created_at: record.created_at,
            updated_at: record.updated_at,
        };
        self.db.with_conn(|conn| {
            diesel::insert_into(templ::workflow_template)
                .values(&row)
                .on_conflict(templ::id)
                .do_update()
                .set((
                    templ::name.eq(&row.name),
                    templ::template_type.eq(&row.template_type),
                    templ::canvas_json.eq(&row.canvas_json),
                    templ::canvas_version.eq(&row.canvas_version),
                    templ::canvas_updated.eq(&row.canvas_updated),
                    templ::updated_at.eq(&row.updated_at),
                ))
                .execute(conn)
                .map(|_| ())
                .map_err(map_err)
        })
    }

    fn upsert_binding(&self, record: WorkflowBindingRecord) -> Result<(), StoreError> {
        let row = NewWorkflowBindingRow {
            account_id: record.account_id,
            template_id: record.template_id,
        };
        self.db.with_conn(|conn| {
            diesel::insert_into(binding::workflow_binding)
                .values(&row)
                .on_conflict(binding::account_id)
                .do_update()
                .set(binding::template_id.eq(&row.template_id))
                .execute(conn)
                .map(|_| ())
                .map_err(map_err)
        })
    }

    fn upsert_stage(&self, record: WorkflowStageRecord) -> Result<(), StoreError> {
        let row = NewWorkflowStageRow {
            template_id: record.template_id,
            id: record.id,
            name: record.name,
            note: record.note,
            ord: record.ord,
            ai_level: record.ai_level,
            x: record.x,
            y: record.y,
            scripts_json: record.scripts_json,
            script_conds_json: record.script_conds_json,
        };
        self.db.with_conn(|conn| {
            diesel::insert_into(stage::workflow_stage)
                .values(&row)
                .on_conflict((stage::template_id, stage::id))
                .do_update()
                .set((
                    stage::name.eq(&row.name),
                    stage::note.eq(&row.note),
                    stage::ord.eq(row.ord),
                    stage::ai_level.eq(&row.ai_level),
                    stage::x.eq(row.x),
                    stage::y.eq(row.y),
                    stage::scripts_json.eq(&row.scripts_json),
                    stage::script_conds_json.eq(&row.script_conds_json),
                ))
                .execute(conn)
                .map(|_| ())
                .map_err(map_err)
        })
    }

    fn upsert_rule(&self, record: WorkflowRuleRecord) -> Result<(), StoreError> {
        let row = NewWorkflowRuleRow {
            id: record.id,
            name: record.name,
            from_stages_json: record.from_stages_json,
            to_stage: record.to_stage,
            trigger_keywords_json: record.trigger_keywords_json,
            trigger_tags_json: record.trigger_tags_json,
            auto_reply: record.auto_reply,
            auto_advance: record.auto_advance,
            reply_script_id: record.reply_script_id,
        };
        self.db.with_conn(|conn| {
            diesel::insert_into(rule::workflow_rule)
                .values(&row)
                .on_conflict(rule::id)
                .do_update()
                .set((
                    rule::name.eq(&row.name),
                    rule::from_stages_json.eq(&row.from_stages_json),
                    rule::to_stage.eq(&row.to_stage),
                    rule::trigger_keywords_json.eq(&row.trigger_keywords_json),
                    rule::trigger_tags_json.eq(&row.trigger_tags_json),
                    rule::auto_reply.eq(row.auto_reply),
                    rule::auto_advance.eq(row.auto_advance),
                    rule::reply_script_id.eq(&row.reply_script_id),
                ))
                .execute(conn)
                .map(|_| ())
                .map_err(map_err)
        })
    }

    fn upsert_script(&self, record: WorkflowScriptRecord) -> Result<(), StoreError> {
        let row = NewWorkflowScriptRow {
            id: record.id,
            stage: record.stage,
            category_l1: record.category_l1,
            category_l2: record.category_l2,
            trigger_text: record.trigger_text,
            description: record.description,
            from_stage: record.from_stage,
            to_stage: record.to_stage,
            tags_json: record.tags_json,
            content: record.content,
            needs_boss_input: record.needs_boss_input,
            boss_input_hint: record.boss_input_hint,
            sort_order: record.sort_order,
            created_at: record.created_at,
            updated_at: record.updated_at,
        };
        self.db.with_conn(|conn| {
            diesel::insert_into(script::workflow_script)
                .values(&row)
                .on_conflict(script::id)
                .do_update()
                .set((
                    script::stage.eq(&row.stage),
                    script::category_l1.eq(&row.category_l1),
                    script::category_l2.eq(&row.category_l2),
                    script::trigger_text.eq(&row.trigger_text),
                    script::description.eq(&row.description),
                    script::from_stage.eq(&row.from_stage),
                    script::to_stage.eq(&row.to_stage),
                    script::tags_json.eq(&row.tags_json),
                    script::content.eq(&row.content),
                    script::needs_boss_input.eq(row.needs_boss_input),
                    script::boss_input_hint.eq(&row.boss_input_hint),
                    script::sort_order.eq(row.sort_order),
                    script::updated_at.eq(&row.updated_at),
                ))
                .execute(conn)
                .map(|_| ())
                .map_err(map_err)
        })
    }
}
