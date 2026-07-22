//! Diesel-backed script snippet store.
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-21

use diesel::prelude::*;
use ports::repository::StoreError;
use ports::workflow::{ScriptSnippetRecord, ScriptSnippetStore, ScriptSnippetWriteInput};
use uuid::Uuid;

use crate::opendesk_db::schema::script_snippet::dsl as snip;
use crate::opendesk_db::{NewScriptSnippetRow, OpendeskDb, ScriptSnippetRow};

/// SQLite implementation of [`ScriptSnippetStore`].
///
/// 作者：Xiaoman
/// 创建时间：2026-07-21
pub struct SqliteScriptSnippetStore {
    db: OpendeskDb,
}

impl SqliteScriptSnippetStore {
    /// Wrap an existing [`OpendeskDb`] handle.
    ///
    /// 作者：Xiaoman
    /// 创建时间：2026-07-21
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

fn now_string() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| (d.as_millis()).to_string())
        .unwrap_or_default()
}

fn row_to_record(row: ScriptSnippetRow) -> ScriptSnippetRecord {
    ScriptSnippetRecord {
        id: row.id,
        source_id: row.source_id,
        title: row.title,
        stage: row.stage,
        trigger_text: row.trigger_text,
        description: row.description,
        from_stage: row.from_stage,
        to_stage: row.to_stage,
        tags_json: row.tags_json,
        body_text: row.body_text,
        category_l1: row.category_l1,
        category_l2: row.category_l2,
        needs_boss_input: row.needs_boss_input,
        boss_input_hint: row.boss_input_hint,
        sort_order: row.sort_order,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

impl ScriptSnippetStore for SqliteScriptSnippetStore {
    fn list(
        &self,
        category_l1: Option<&str>,
        category_l2: Option<&str>,
        query: Option<&str>,
    ) -> Result<Vec<ScriptSnippetRecord>, StoreError> {
        self.db.with_conn(|conn| {
            let mut q = snip::script_snippet
                .order((snip::sort_order.asc(), snip::created_at.asc()))
                .into_boxed();

            if let Some(l1) = category_l1 {
                q = q.filter(snip::category_l1.eq(l1));
            }
            if let Some(l2) = category_l2 {
                q = q.filter(snip::category_l2.eq(l2));
            }
            if let Some(kw) = query {
                let pattern = format!("%{kw}%");
                q = q.filter(
                    snip::title
                        .like(pattern.clone())
                        .or(snip::trigger_text.like(pattern.clone()))
                        .or(snip::body_text.like(pattern)),
                );
            }

            q.select(ScriptSnippetRow::as_select())
                .load::<ScriptSnippetRow>(conn)
                .map(|rows| rows.into_iter().map(row_to_record).collect())
                .map_err(map_err)
        })
    }

    fn get(&self, id: &str) -> Result<ScriptSnippetRecord, StoreError> {
        self.db.with_conn(|conn| {
            snip::script_snippet
                .filter(snip::id.eq(id))
                .select(ScriptSnippetRow::as_select())
                .first::<ScriptSnippetRow>(conn)
                .map(row_to_record)
                .map_err(map_err)
        })
    }

    fn save(&self, input: ScriptSnippetWriteInput) -> Result<ScriptSnippetRecord, StoreError> {
        let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let now = now_string();
        let row = NewScriptSnippetRow {
            id: id.clone(),
            source_id: id.clone(),
            title: input.title,
            stage: input.stage,
            trigger_text: input.trigger_text,
            description: input.description,
            from_stage: input.from_stage,
            to_stage: input.to_stage,
            tags_json: input.tags_json,
            body_text: input.body_text,
            category_l1: input.category_l1,
            category_l2: input.category_l2,
            needs_boss_input: input.needs_boss_input,
            boss_input_hint: input.boss_input_hint,
            sort_order: input.sort_order,
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        self.db.with_conn(|conn| {
            let exists = snip::script_snippet
                .filter(snip::id.eq(&id))
                .count()
                .get_result::<i64>(conn)
                .map_err(map_err)?
                > 0;

            if exists {
                diesel::update(snip::script_snippet.filter(snip::id.eq(&id)))
                    .set((
                        snip::title.eq(&row.title),
                        snip::stage.eq(&row.stage),
                        snip::trigger_text.eq(&row.trigger_text),
                        snip::description.eq(&row.description),
                        snip::from_stage.eq(&row.from_stage),
                        snip::to_stage.eq(&row.to_stage),
                        snip::tags_json.eq(&row.tags_json),
                        snip::body_text.eq(&row.body_text),
                        snip::category_l1.eq(&row.category_l1),
                        snip::category_l2.eq(&row.category_l2),
                        snip::needs_boss_input.eq(row.needs_boss_input),
                        snip::boss_input_hint.eq(&row.boss_input_hint),
                        snip::sort_order.eq(row.sort_order),
                        snip::updated_at.eq(&row.updated_at),
                    ))
                    .execute(conn)
                    .map_err(map_err)?;
            } else {
                diesel::insert_into(snip::script_snippet)
                    .values(&row)
                    .execute(conn)
                    .map_err(map_err)?;
            }

            snip::script_snippet
                .filter(snip::id.eq(&id))
                .select(ScriptSnippetRow::as_select())
                .first::<ScriptSnippetRow>(conn)
                .map(row_to_record)
                .map_err(map_err)
        })
    }

    fn delete(&self, id: &str) -> Result<(), StoreError> {
        self.db.with_conn(|conn| {
            diesel::delete(snip::script_snippet.filter(snip::id.eq(id)))
                .execute(conn)
                .map_err(map_err)?;
            Ok(())
        })
    }
}
