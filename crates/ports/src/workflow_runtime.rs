//! Workflow Runtime 检查点 Port。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-23

use crate::repository::StoreError;

/// 工作流实例落库行。
///
/// @author Xiaoman
/// @created 2026-07-23
#[derive(Debug, Clone, PartialEq)]
pub struct WfRtInstanceRecord {
    pub instance_id: String,
    pub definition_id: Option<String>,
    pub definition_json: String,
    pub state: String,
    pub context_json: String,
    pub context_version: i64,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub heartbeat_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

/// 节点实例落库行。
///
/// @author Xiaoman
/// @created 2026-07-23
#[derive(Debug, Clone, PartialEq)]
pub struct WfRtNodeRecord {
    pub instance_id: String,
    pub node_id: String,
    pub node_type: String,
    pub state: String,
    pub attempt: i64,
    pub max_retry: i64,
    pub retry_state_json: String,
    pub input_json: Option<String>,
    pub output_json: Option<String>,
    pub error_message: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub duration_ms: Option<i64>,
}

/// 日志行。
///
/// @author Xiaoman
/// @created 2026-07-23
#[derive(Debug, Clone, PartialEq)]
pub struct WfRtLogRecord {
    pub id: String,
    pub instance_id: String,
    pub node_id: Option<String>,
    pub level: String,
    pub event_kind: String,
    pub payload_json: String,
    pub created_at: String,
}

/// 单节点进度事务提交载荷。
///
/// @author Xiaoman
/// @created 2026-07-23
#[derive(Debug, Clone)]
pub struct NodeProgressCommit {
    pub instance: WfRtInstanceRecord,
    pub node: WfRtNodeRecord,
    pub log: WfRtLogRecord,
}

/// 工作流检查点存储。
///
/// @author Xiaoman
/// @created 2026-07-23
pub trait CheckpointStore: Send + Sync {
    /// 插入新实例及全部 Pending 节点。
    fn create_instance(
        &self,
        instance: &WfRtInstanceRecord,
        nodes: &[WfRtNodeRecord],
    ) -> Result<(), StoreError>;

    /// 事务提交节点进度（instance + node + log）。
    fn commit_node_progress(&self, commit: &NodeProgressCommit) -> Result<(), StoreError>;

    /// 更新实例状态 / context / heartbeat。
    fn update_instance(&self, instance: &WfRtInstanceRecord) -> Result<(), StoreError>;

    /// 按 id 读取实例。
    fn get_instance(&self, instance_id: &str) -> Result<Option<WfRtInstanceRecord>, StoreError>;

    /// 读取实例全部节点。
    fn list_nodes(&self, instance_id: &str) -> Result<Vec<WfRtNodeRecord>, StoreError>;

    /// 列出可恢复实例（running/pausing/paused/failing/cancelling）。
    fn list_recoverable(&self) -> Result<Vec<WfRtInstanceRecord>, StoreError>;

    /// 追加日志。
    fn append_log(&self, log: &WfRtLogRecord) -> Result<(), StoreError>;
}
