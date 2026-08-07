//! Checkpoint 网关：Runtime 类型 ↔ Port 记录。
//!
//! 作者：coisini
//! 创建时间：2026-07-23

use crate::context::WorkflowContext;
use crate::definition::{NodeSpec, WorkflowDefinition};
use crate::error::WorkflowError;
use crate::id::{InstanceId, NodeId};
use crate::state::{NodeState, RetryState, WorkflowState};
use ports::repository::StoreError;
use ports::workflow_runtime::{
    CheckpointStore, NodeProgressCommit, WfRtInstanceRecord, WfRtLogRecord, WfRtNodeRecord,
};
use std::sync::Arc;
use uuid::Uuid;

/// 将 StoreError 映射为 WorkflowError。
///
/// @author coisini
/// @created 2026-07-23
///
/// @param error - 存储错误
/// @returns Runtime 错误
pub fn map_store_error(error: StoreError) -> WorkflowError {
    WorkflowError::persistence(error.to_string())
}

/// 当前时间 Unix 毫秒串（命名沿历史保留，实为毫秒串）。
///
/// @author coisini
/// @created 2026-07-23
///
/// @returns 时间字符串
pub fn now_rfc3339() -> String {
    common::tools::time::now_millis_string()
}

/// 当前 Unix 毫秒。
///
/// @author coisini
/// @created 2026-07-23
///
/// @returns 毫秒
pub fn now_ms() -> i64 {
    common::tools::time::now_millis_i64()
}

/// 检查点读写门面。
///
/// @author coisini
/// @created 2026-07-23
#[derive(Clone)]
pub struct CheckpointGateway {
    store: Arc<dyn CheckpointStore>,
}

impl CheckpointGateway {
    /// 构造。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @param store - Port 实现
    /// @returns 网关
    pub fn new(store: Arc<dyn CheckpointStore>) -> Self {
        Self { store }
    }

    /// 底层 store。
    ///
    /// @author coisini
    /// @created 2026-07-23
    ///
    /// @returns Arc
    pub fn store(&self) -> Arc<dyn CheckpointStore> {
        Arc::clone(&self.store)
    }

    /// 创建实例。
    ///
    /// @author coisini
    /// @created 2026-07-23
    pub fn create_instance(
        &self,
        instance_id: &InstanceId,
        definition: &WorkflowDefinition,
        state: WorkflowState,
        context: &WorkflowContext,
        node_specs: &[&NodeSpec],
        initial_node_states: &[(NodeId, NodeState)],
    ) -> Result<(), WorkflowError> {
        let now = now_rfc3339();
        let definition_json = serde_json::to_string(definition).map_err(|error| {
            WorkflowError::persistence(format!("serialize definition: {error}"))
        })?;
        let context_json = serde_json::to_string(&context.to_value())
            .map_err(|error| WorkflowError::persistence(format!("serialize context: {error}")))?;

        let instance = WfRtInstanceRecord {
            instance_id: instance_id.as_str().to_string(),
            definition_id: Some(definition.id.as_str().to_string()),
            definition_json,
            state: state.as_str().to_string(),
            context_json,
            context_version: context.version() as i64,
            error_code: None,
            error_message: None,
            heartbeat_at: Some(now.clone()),
            created_at: now.clone(),
            updated_at: now.clone(),
            started_at: Some(now.clone()),
            finished_at: None,
        };

        let mut nodes = Vec::new();
        for spec in node_specs {
            let state = match initial_node_states.iter().find(|(id, _)| id == &spec.id) {
                Some((_, state)) => *state,
                None => NodeState::Pending,
            };
            nodes.push(WfRtNodeRecord {
                instance_id: instance_id.as_str().to_string(),
                node_id: spec.id.as_str().to_string(),
                node_type: spec.node_type.as_str().to_string(),
                state: state.as_str().to_string(),
                attempt: 0,
                max_retry: spec.retry.max_retry as i64,
                retry_state_json: RetryState::NotRetrying.to_json_string(),
                input_json: None,
                output_json: None,
                error_message: None,
                started_at: None,
                finished_at: None,
                duration_ms: None,
            });
        }

        self.store
            .create_instance(&instance, &nodes)
            .map_err(map_store_error)
    }

    /// 提交节点进度。
    ///
    /// @author coisini
    /// @created 2026-07-23
    pub fn commit_node_progress(
        &self,
        instance: &WfRtInstanceRecord,
        node: &WfRtNodeRecord,
        event_kind: &str,
        payload_json: &str,
    ) -> Result<(), WorkflowError> {
        let log = WfRtLogRecord {
            id: Uuid::new_v4().to_string(),
            instance_id: instance.instance_id.clone(),
            node_id: Some(node.node_id.clone()),
            level: "info".to_string(),
            event_kind: event_kind.to_string(),
            payload_json: payload_json.to_string(),
            created_at: now_rfc3339(),
        };
        self.store
            .commit_node_progress(&NodeProgressCommit {
                instance: instance.clone(),
                node: node.clone(),
                log,
            })
            .map_err(map_store_error)
    }

    /// 更新实例。
    ///
    /// @author coisini
    /// @created 2026-07-23
    pub fn update_instance(&self, instance: &WfRtInstanceRecord) -> Result<(), WorkflowError> {
        self.store
            .update_instance(instance)
            .map_err(map_store_error)
    }

    /// 读取实例。
    ///
    /// @author coisini
    /// @created 2026-07-23
    pub fn get_instance(
        &self,
        instance_id: &InstanceId,
    ) -> Result<Option<WfRtInstanceRecord>, WorkflowError> {
        self.store
            .get_instance(instance_id.as_str())
            .map_err(map_store_error)
    }

    /// 列出节点。
    ///
    /// @author coisini
    /// @created 2026-07-23
    pub fn list_nodes(
        &self,
        instance_id: &InstanceId,
    ) -> Result<Vec<WfRtNodeRecord>, WorkflowError> {
        self.store
            .list_nodes(instance_id.as_str())
            .map_err(map_store_error)
    }

    /// 可恢复列表。
    ///
    /// @author coisini
    /// @created 2026-07-23
    pub fn list_recoverable(&self) -> Result<Vec<WfRtInstanceRecord>, WorkflowError> {
        self.store.list_recoverable().map_err(map_store_error)
    }
}

/// 从 JSON 重建 Context。
///
/// @author coisini
/// @created 2026-07-23
pub fn context_from_record(record: &WfRtInstanceRecord) -> Result<WorkflowContext, WorkflowError> {
    let value: serde_json::Value = serde_json::from_str(&record.context_json)
        .map_err(|error| WorkflowError::persistence(format!("parse context: {error}")))?;
    Ok(WorkflowContext::from_value(
        value,
        record.context_version as u64,
    ))
}

/// 从 JSON 重建 Definition。
///
/// @author coisini
/// @created 2026-07-23
pub fn definition_from_record(
    record: &WfRtInstanceRecord,
) -> Result<WorkflowDefinition, WorkflowError> {
    serde_json::from_str(&record.definition_json)
        .map_err(|error| WorkflowError::persistence(format!("parse definition: {error}")))
}
