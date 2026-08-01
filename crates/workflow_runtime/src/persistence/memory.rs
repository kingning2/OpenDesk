//! 内存 CheckpointStore（单测 / 无 DB 场景）。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-23

use ports::repository::StoreError;
use ports::workflow_runtime::{
    CheckpointStore, NodeProgressCommit, WfRtInstanceRecord, WfRtLogRecord, WfRtNodeRecord,
};
use std::collections::HashMap;
use std::sync::Mutex;

/// 进程内检查点实现。
///
/// @author Xiaoman
/// @created 2026-07-23
#[derive(Default)]
pub struct InMemoryCheckpointStore {
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    instances: HashMap<String, WfRtInstanceRecord>,
    nodes: HashMap<String, Vec<WfRtNodeRecord>>,
    logs: Vec<WfRtLogRecord>,
}

impl InMemoryCheckpointStore {
    /// 新建空存储。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @returns 存储
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Inner>, StoreError> {
        self.inner
            .lock()
            .map_err(|error| StoreError::Unavailable(format!("lock poisoned: {error}")))
    }
}

impl CheckpointStore for InMemoryCheckpointStore {
    fn create_instance(
        &self,
        instance: &WfRtInstanceRecord,
        nodes: &[WfRtNodeRecord],
    ) -> Result<(), StoreError> {
        let mut guard = self.lock()?;
        match guard.instances.contains_key(&instance.instance_id) {
            true => {
                return Err(StoreError::Conflict(format!(
                    "instance exists: {}",
                    instance.instance_id
                )));
            }
            false => {}
        }
        guard
            .instances
            .insert(instance.instance_id.clone(), instance.clone());
        guard
            .nodes
            .insert(instance.instance_id.clone(), nodes.to_vec());
        Ok(())
    }

    fn commit_node_progress(&self, commit: &NodeProgressCommit) -> Result<(), StoreError> {
        let mut guard = self.lock()?;
        guard
            .instances
            .insert(commit.instance.instance_id.clone(), commit.instance.clone());
        let nodes = guard
            .nodes
            .entry(commit.instance.instance_id.clone())
            .or_default();
        match nodes
            .iter()
            .position(|row| row.node_id == commit.node.node_id)
        {
            Some(index) => nodes[index] = commit.node.clone(),
            None => nodes.push(commit.node.clone()),
        }
        guard.logs.push(commit.log.clone());
        Ok(())
    }

    fn update_instance(&self, instance: &WfRtInstanceRecord) -> Result<(), StoreError> {
        let mut guard = self.lock()?;
        match guard.instances.contains_key(&instance.instance_id) {
            false => return Err(StoreError::NotFound),
            true => {
                guard
                    .instances
                    .insert(instance.instance_id.clone(), instance.clone());
            }
        }
        Ok(())
    }

    fn get_instance(&self, instance_id: &str) -> Result<Option<WfRtInstanceRecord>, StoreError> {
        let guard = self.lock()?;
        Ok(guard.instances.get(instance_id).cloned())
    }

    fn list_nodes(&self, instance_id: &str) -> Result<Vec<WfRtNodeRecord>, StoreError> {
        let guard = self.lock()?;
        Ok(guard.nodes.get(instance_id).cloned().unwrap_or_default())
    }

    fn list_recoverable(&self) -> Result<Vec<WfRtInstanceRecord>, StoreError> {
        let guard = self.lock()?;
        let mut out = Vec::new();
        for instance in guard.instances.values() {
            match instance.state.as_str() {
                "running" | "pausing" | "paused" | "failing" | "cancelling" => {
                    out.push(instance.clone());
                }
                _ => {}
            }
        }
        Ok(out)
    }

    fn append_log(&self, log: &WfRtLogRecord) -> Result<(), StoreError> {
        let mut guard = self.lock()?;
        guard.logs.push(log.clone());
        Ok(())
    }
}
