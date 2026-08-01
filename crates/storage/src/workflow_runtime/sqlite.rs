//! SQLite CheckpointStore。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-23

use diesel::prelude::*;
use ports::repository::StoreError;
use ports::workflow_runtime::{
    CheckpointStore, NodeProgressCommit, WfRtInstanceRecord, WfRtLogRecord, WfRtNodeRecord,
};

use crate::opendesk_db::schema::{wf_rt_instance, wf_rt_log, wf_rt_node_instance};
use crate::opendesk_db::OpendeskDb;

/// Diesel 实现的检查点存储。
///
/// @author Xiaoman
/// @created 2026-07-23
pub struct SqliteCheckpointStore {
    db: OpendeskDb,
}

impl SqliteCheckpointStore {
    /// 包装数据库句柄。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @param db - opendesk.db
    /// @returns Store
    pub fn new(db: OpendeskDb) -> Self {
        Self { db }
    }
}

fn map_err(error: diesel::result::Error) -> StoreError {
    match error {
        diesel::result::Error::NotFound => StoreError::NotFound,
        other => StoreError::Unavailable(other.to_string()),
    }
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = wf_rt_instance)]
struct InstanceInsert<'a> {
    instance_id: &'a str,
    definition_id: Option<&'a str>,
    definition_json: &'a str,
    state: &'a str,
    context_json: &'a str,
    context_version: i64,
    error_code: Option<&'a str>,
    error_message: Option<&'a str>,
    heartbeat_at: Option<&'a str>,
    created_at: &'a str,
    updated_at: &'a str,
    started_at: Option<&'a str>,
    finished_at: Option<&'a str>,
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = wf_rt_node_instance)]
struct NodeInsert<'a> {
    instance_id: &'a str,
    node_id: &'a str,
    node_type: &'a str,
    state: &'a str,
    attempt: i64,
    max_retry: i64,
    retry_state_json: &'a str,
    input_json: Option<&'a str>,
    output_json: Option<&'a str>,
    error_message: Option<&'a str>,
    started_at: Option<&'a str>,
    finished_at: Option<&'a str>,
    duration_ms: Option<i64>,
}

#[derive(Insertable)]
#[diesel(table_name = wf_rt_log)]
struct LogInsert<'a> {
    id: &'a str,
    instance_id: &'a str,
    node_id: Option<&'a str>,
    level: &'a str,
    event_kind: &'a str,
    payload_json: &'a str,
    created_at: &'a str,
}

#[derive(Queryable)]
struct InstanceRow {
    instance_id: String,
    definition_id: Option<String>,
    definition_json: String,
    state: String,
    context_json: String,
    context_version: i64,
    error_code: Option<String>,
    error_message: Option<String>,
    heartbeat_at: Option<String>,
    created_at: String,
    updated_at: String,
    started_at: Option<String>,
    finished_at: Option<String>,
}

#[derive(Queryable)]
struct NodeRow {
    instance_id: String,
    node_id: String,
    node_type: String,
    state: String,
    attempt: i64,
    max_retry: i64,
    retry_state_json: String,
    input_json: Option<String>,
    output_json: Option<String>,
    error_message: Option<String>,
    started_at: Option<String>,
    finished_at: Option<String>,
    duration_ms: Option<i64>,
}

fn instance_from_row(row: InstanceRow) -> WfRtInstanceRecord {
    WfRtInstanceRecord {
        instance_id: row.instance_id,
        definition_id: row.definition_id,
        definition_json: row.definition_json,
        state: row.state,
        context_json: row.context_json,
        context_version: row.context_version,
        error_code: row.error_code,
        error_message: row.error_message,
        heartbeat_at: row.heartbeat_at,
        created_at: row.created_at,
        updated_at: row.updated_at,
        started_at: row.started_at,
        finished_at: row.finished_at,
    }
}

fn node_from_row(row: NodeRow) -> WfRtNodeRecord {
    WfRtNodeRecord {
        instance_id: row.instance_id,
        node_id: row.node_id,
        node_type: row.node_type,
        state: row.state,
        attempt: row.attempt,
        max_retry: row.max_retry,
        retry_state_json: row.retry_state_json,
        input_json: row.input_json,
        output_json: row.output_json,
        error_message: row.error_message,
        started_at: row.started_at,
        finished_at: row.finished_at,
        duration_ms: row.duration_ms,
    }
}

fn instance_insert<'a>(record: &'a WfRtInstanceRecord) -> InstanceInsert<'a> {
    InstanceInsert {
        instance_id: &record.instance_id,
        definition_id: record.definition_id.as_deref(),
        definition_json: &record.definition_json,
        state: &record.state,
        context_json: &record.context_json,
        context_version: record.context_version,
        error_code: record.error_code.as_deref(),
        error_message: record.error_message.as_deref(),
        heartbeat_at: record.heartbeat_at.as_deref(),
        created_at: &record.created_at,
        updated_at: &record.updated_at,
        started_at: record.started_at.as_deref(),
        finished_at: record.finished_at.as_deref(),
    }
}

fn node_insert<'a>(record: &'a WfRtNodeRecord) -> NodeInsert<'a> {
    NodeInsert {
        instance_id: &record.instance_id,
        node_id: &record.node_id,
        node_type: &record.node_type,
        state: &record.state,
        attempt: record.attempt,
        max_retry: record.max_retry,
        retry_state_json: &record.retry_state_json,
        input_json: record.input_json.as_deref(),
        output_json: record.output_json.as_deref(),
        error_message: record.error_message.as_deref(),
        started_at: record.started_at.as_deref(),
        finished_at: record.finished_at.as_deref(),
        duration_ms: record.duration_ms,
    }
}

impl CheckpointStore for SqliteCheckpointStore {
    fn create_instance(
        &self,
        instance: &WfRtInstanceRecord,
        nodes: &[WfRtNodeRecord],
    ) -> Result<(), StoreError> {
        self.db.with_conn(|conn| {
            conn.transaction::<(), diesel::result::Error, _>(|conn| {
                diesel::insert_into(wf_rt_instance::table)
                    .values(instance_insert(instance))
                    .execute(conn)?;
                for node in nodes {
                    diesel::insert_into(wf_rt_node_instance::table)
                        .values(node_insert(node))
                        .execute(conn)?;
                }
                Ok(())
            })
            .map_err(map_err)
        })
    }

    fn commit_node_progress(&self, commit: &NodeProgressCommit) -> Result<(), StoreError> {
        self.db.with_conn(|conn| {
            conn.transaction::<(), diesel::result::Error, _>(|conn| {
                diesel::insert_into(wf_rt_instance::table)
                    .values(instance_insert(&commit.instance))
                    .on_conflict(wf_rt_instance::instance_id)
                    .do_update()
                    .set(instance_insert(&commit.instance))
                    .execute(conn)?;

                diesel::insert_into(wf_rt_node_instance::table)
                    .values(node_insert(&commit.node))
                    .on_conflict((
                        wf_rt_node_instance::instance_id,
                        wf_rt_node_instance::node_id,
                    ))
                    .do_update()
                    .set(node_insert(&commit.node))
                    .execute(conn)?;

                diesel::insert_into(wf_rt_log::table)
                    .values(LogInsert {
                        id: &commit.log.id,
                        instance_id: &commit.log.instance_id,
                        node_id: commit.log.node_id.as_deref(),
                        level: &commit.log.level,
                        event_kind: &commit.log.event_kind,
                        payload_json: &commit.log.payload_json,
                        created_at: &commit.log.created_at,
                    })
                    .execute(conn)?;
                Ok(())
            })
            .map_err(map_err)
        })
    }

    fn update_instance(&self, instance: &WfRtInstanceRecord) -> Result<(), StoreError> {
        self.db.with_conn(|conn| {
            let updated = diesel::update(
                wf_rt_instance::table.filter(wf_rt_instance::instance_id.eq(&instance.instance_id)),
            )
            .set(instance_insert(instance))
            .execute(conn)
            .map_err(map_err)?;
            match updated {
                0 => Err(StoreError::NotFound),
                _ => Ok(()),
            }
        })
    }

    fn get_instance(&self, instance_id: &str) -> Result<Option<WfRtInstanceRecord>, StoreError> {
        self.db.with_conn(|conn| {
            let row = wf_rt_instance::table
                .filter(wf_rt_instance::instance_id.eq(instance_id))
                .first::<InstanceRow>(conn)
                .optional()
                .map_err(map_err)?;
            Ok(row.map(instance_from_row))
        })
    }

    fn list_nodes(&self, instance_id: &str) -> Result<Vec<WfRtNodeRecord>, StoreError> {
        self.db.with_conn(|conn| {
            let rows = wf_rt_node_instance::table
                .filter(wf_rt_node_instance::instance_id.eq(instance_id))
                .load::<NodeRow>(conn)
                .map_err(map_err)?;
            Ok(rows.into_iter().map(node_from_row).collect())
        })
    }

    fn list_recoverable(&self) -> Result<Vec<WfRtInstanceRecord>, StoreError> {
        self.db.with_conn(|conn| {
            let rows = wf_rt_instance::table
                .filter(wf_rt_instance::state.eq_any([
                    "running",
                    "pausing",
                    "paused",
                    "failing",
                    "cancelling",
                ]))
                .load::<InstanceRow>(conn)
                .map_err(map_err)?;
            Ok(rows.into_iter().map(instance_from_row).collect())
        })
    }

    fn append_log(&self, log: &WfRtLogRecord) -> Result<(), StoreError> {
        self.db.with_conn(|conn| {
            diesel::insert_into(wf_rt_log::table)
                .values(LogInsert {
                    id: &log.id,
                    instance_id: &log.instance_id,
                    node_id: log.node_id.as_deref(),
                    level: &log.level,
                    event_kind: &log.event_kind,
                    payload_json: &log.payload_json,
                    created_at: &log.created_at,
                })
                .execute(conn)
                .map_err(map_err)?;
            Ok(())
        })
    }
}
