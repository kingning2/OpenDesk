//! RecoveryService：扫描 Running 等并支持 Resume。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-23

use crate::dag::DagBuilder;
use crate::error::WorkflowError;
use crate::id::InstanceId;
use crate::persistence::{definition_from_record, now_rfc3339, CheckpointGateway};
use crate::scheduler::{handle_from_checkpoint, Scheduler, SchedulerHandle};
use crate::state::WorkflowState;
use ports::workflow_runtime::WfRtInstanceRecord;
use std::sync::Arc;

/// 恢复服务。
///
/// @author Xiaoman
/// @created 2026-07-23
pub struct RecoveryService {
    checkpoint: CheckpointGateway,
    scheduler: Arc<Scheduler>,
}

impl RecoveryService {
    /// 构造。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    pub fn new(checkpoint: CheckpointGateway, scheduler: Arc<Scheduler>) -> Self {
        Self {
            checkpoint,
            scheduler,
        }
    }

    /// 扫描可恢复实例（供 UI 展示）。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @returns 实例列表
    pub fn list_recoverable(&self) -> Result<Vec<WfRtInstanceRecord>, WorkflowError> {
        self.checkpoint.list_recoverable()
    }

    /// 心跳刷新（可选周期调用）。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    pub fn touch_heartbeat(&self, instance_id: &InstanceId) -> Result<(), WorkflowError> {
        let mut record = match self.checkpoint.get_instance(instance_id)? {
            Some(record) => record,
            None => {
                return Err(WorkflowError::InstanceNotFound {
                    instance_id: instance_id.to_string(),
                });
            }
        };
        record.heartbeat_at = Some(now_rfc3339());
        record.updated_at = now_rfc3339();
        self.checkpoint.update_instance(&record)
    }

    /// 恢复并继续跑到 idle。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @param instance_id - 实例
    /// @returns 最终状态
    pub async fn resume(&self, instance_id: &InstanceId) -> Result<WorkflowState, WorkflowError> {
        let mut handle = self.load_handle(instance_id)?;
        match handle.workflow_state {
            WorkflowState::Paused => {
                handle.workflow_state = WorkflowState::Running;
            }
            WorkflowState::Running
            | WorkflowState::Pausing
            | WorkflowState::Failing
            | WorkflowState::Cancelling => {
                handle.workflow_state = WorkflowState::Running;
            }
            other => {
                return Err(WorkflowError::NotAllowed {
                    message: format!("cannot resume from state {other}"),
                });
            }
        }
        self.scheduler.run_until_idle(&mut handle).await
    }

    /// 加载句柄。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    pub fn load_handle(&self, instance_id: &InstanceId) -> Result<SchedulerHandle, WorkflowError> {
        let record = match self.checkpoint.get_instance(instance_id)? {
            Some(record) => record,
            None => {
                return Err(WorkflowError::InstanceNotFound {
                    instance_id: instance_id.to_string(),
                });
            }
        };
        let definition = definition_from_record(&record)?;
        let graph = DagBuilder::build(&definition)?;
        handle_from_checkpoint(&self.checkpoint, instance_id, graph, definition)
    }
}
