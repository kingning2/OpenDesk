//! Scheduler 引擎主循环。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-23

use super::concurrency::SchedulerConfig;
use super::ready::compute_ready_nodes;
use super::retry::compute_retry_delay;
use crate::context::WorkflowContext;
use crate::dag::WorkflowGraph;
use crate::definition::{NodeType, WorkflowDefinition};
use crate::error::WorkflowError;
use crate::event::{WorkflowEvent, WorkflowEventBus};
use crate::executor::{ExecuteInput, ExecutorRegistry};
use crate::id::{InstanceId, NodeId};
use crate::persistence::{context_from_record, now_ms, now_rfc3339, CheckpointGateway};
use crate::state::{
    transition_node_state, transition_workflow_state, NodeState, NodeTransition, RetryState,
    WorkflowState, WorkflowTransition,
};
use ports::workflow_runtime::{WfRtInstanceRecord, WfRtNodeRecord};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// 运行中实例句柄（内存态）。
///
/// @author Xiaoman
/// @created 2026-07-23
pub struct SchedulerHandle {
    pub instance_id: InstanceId,
    pub definition: WorkflowDefinition,
    pub graph: WorkflowGraph,
    pub workflow_state: WorkflowState,
    pub node_states: HashMap<NodeId, NodeState>,
    pub attempts: HashMap<NodeId, u32>,
    pub retry_states: HashMap<NodeId, RetryState>,
    pub context: WorkflowContext,
    pub cancel_requested: bool,
    pub pause_requested: bool,
}

/// Scheduler：无业务 I/O，只调度 Registry 中的 Executor。
///
/// @author Xiaoman
/// @created 2026-07-23
pub struct Scheduler {
    registry: Arc<ExecutorRegistry>,
    checkpoint: CheckpointGateway,
    event_bus: Arc<dyn WorkflowEventBus>,
    config: SchedulerConfig,
}

impl Scheduler {
    /// 构造。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    pub fn new(
        registry: Arc<ExecutorRegistry>,
        checkpoint: CheckpointGateway,
        event_bus: Arc<dyn WorkflowEventBus>,
        config: SchedulerConfig,
    ) -> Self {
        Self {
            registry,
            checkpoint,
            event_bus,
            config,
        }
    }

    /// 跑完一个实例（阻塞直到终态或 pause drained）。
    ///
    /// @author Xiaoman
    /// @created 2026-07-23
    ///
    /// @param handle - 可变句柄
    /// @returns 最终工作流状态
    pub async fn run_until_idle(
        &self,
        handle: &mut SchedulerHandle,
    ) -> Result<WorkflowState, WorkflowError> {
        loop {
            match handle.workflow_state {
                WorkflowState::Completed
                | WorkflowState::Failed
                | WorkflowState::Cancelled
                | WorkflowState::Paused => return Ok(handle.workflow_state),
                WorkflowState::Pending
                | WorkflowState::Running
                | WorkflowState::Pausing
                | WorkflowState::Completing
                | WorkflowState::Failing
                | WorkflowState::Cancelling => {}
            }

            self.promote_retry_ready(handle)?;

            match handle.cancel_requested {
                true => {
                    self.apply_workflow_transition(handle, WorkflowTransition::RequestCancel)?;
                    self.cancel_active_nodes(handle)?;
                    self.apply_workflow_transition(handle, WorkflowTransition::FinishCancel)?;
                    self.event_bus.publish(WorkflowEvent::WorkflowCancelled {
                        instance_id: handle.instance_id.clone(),
                    })?;
                    self.persist_instance(handle, None, None)?;
                    return Ok(handle.workflow_state);
                }
                false => {}
            }

            match handle.pause_requested {
                true => match handle.workflow_state {
                    WorkflowState::Running => {
                        self.apply_workflow_transition(handle, WorkflowTransition::RequestPause)?;
                    }
                    WorkflowState::Pausing => {
                        let in_flight = handle
                            .node_states
                            .values()
                            .filter(|state| matches!(state, NodeState::Running))
                            .count();
                        match in_flight {
                            0 => {
                                self.apply_workflow_transition(
                                    handle,
                                    WorkflowTransition::PauseDrained,
                                )?;
                                self.event_bus.publish(WorkflowEvent::WorkflowPaused {
                                    instance_id: handle.instance_id.clone(),
                                })?;
                                self.persist_instance(handle, None, None)?;
                                return Ok(handle.workflow_state);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                },
                false => {}
            }

            let ready = compute_ready_nodes(&handle.graph, &handle.node_states);
            let mut dispatched = 0usize;
            let in_flight = handle
                .node_states
                .values()
                .filter(|state| matches!(state, NodeState::Running))
                .count();

            for node_id in ready {
                match in_flight + dispatched < self.config.max_in_flight {
                    true => {}
                    false => break,
                }
                match handle.workflow_state {
                    WorkflowState::Pausing | WorkflowState::Cancelling => break,
                    _ => {}
                }

                let current = match handle.node_states.get(&node_id) {
                    Some(state) => *state,
                    None => NodeState::Pending,
                };
                let ready_state = match current {
                    NodeState::Ready => NodeState::Ready,
                    other => transition_node_state(other, NodeTransition::MakeReady)?,
                };
                handle.node_states.insert(node_id.clone(), ready_state);
                let running = transition_node_state(ready_state, NodeTransition::Dispatch)?;
                handle.node_states.insert(node_id.clone(), running);

                self.event_bus.publish(WorkflowEvent::NodeStarted {
                    instance_id: handle.instance_id.clone(),
                    node_id: node_id.clone(),
                })?;

                self.execute_one(handle, &node_id).await?;
                dispatched = dispatched.saturating_add(1);
            }

            match dispatched {
                0 => {
                    self.finish_if_possible(handle)?;
                    match handle.workflow_state.is_terminal()
                        || handle.workflow_state == WorkflowState::Paused
                    {
                        true => return Ok(handle.workflow_state),
                        false => {
                            // 可能在 RetryWaiting
                            let waiting = handle
                                .node_states
                                .values()
                                .any(|state| matches!(state, NodeState::RetryWaiting));
                            match waiting {
                                true => tokio::time::sleep(Duration::from_millis(50)).await,
                                false => {
                                    self.finish_if_possible(handle)?;
                                    return Ok(handle.workflow_state);
                                }
                            }
                        }
                    }
                }
                _ => {
                    self.finish_if_possible(handle)?;
                }
            }
        }
    }

    async fn execute_one(
        &self,
        handle: &mut SchedulerHandle,
        node_id: &NodeId,
    ) -> Result<(), WorkflowError> {
        let spec = match handle.graph.node(node_id) {
            Some(spec) => spec.clone(),
            None => {
                return Err(WorkflowError::NodeNotFound {
                    node_id: node_id.to_string(),
                });
            }
        };

        let attempt = handle
            .attempts
            .get(node_id)
            .copied()
            .unwrap_or(0)
            .saturating_add(1);
        handle.attempts.insert(node_id.clone(), attempt);

        let executor = self.registry.get(spec.node_type)?;
        let started_ms = now_ms();
        let input = ExecuteInput {
            node_id,
            config: &spec.config,
            context: &handle.context,
            attempt,
        };

        let result = executor.execute(input).await;
        let finished_ms = now_ms();
        let duration_ms = finished_ms.saturating_sub(started_ms);

        match result {
            Ok(output) => {
                handle.context.apply_patches(&output.context_patches)?;
                let next = transition_node_state(NodeState::Running, NodeTransition::Succeed)?;
                handle.node_states.insert(node_id.clone(), next);
                handle
                    .retry_states
                    .insert(node_id.clone(), RetryState::NotRetrying);

                self.apply_branch_skips(handle, node_id, output.branch.as_deref())?;

                self.event_bus.publish(WorkflowEvent::NodeCompleted {
                    instance_id: handle.instance_id.clone(),
                    node_id: node_id.clone(),
                    state: next,
                })?;
                match output.context_patches.is_empty() {
                    false => {
                        self.event_bus.publish(WorkflowEvent::ContextChanged {
                            instance_id: handle.instance_id.clone(),
                            version: handle.context.version(),
                        })?;
                    }
                    true => {}
                }

                self.persist_node_success(handle, &spec, attempt, duration_ms, &output.message)?;
            }
            Err(error) => {
                let max_retry = spec.retry.max_retry;
                let failed_attempts = attempt;
                match failed_attempts <= max_retry {
                    true => {
                        let delay = compute_retry_delay(&spec.retry, failed_attempts);
                        let next_at = now_ms() + delay.as_millis() as i64;
                        let next = transition_node_state(
                            NodeState::Running,
                            NodeTransition::ScheduleRetry,
                        )?;
                        handle.node_states.insert(node_id.clone(), next);
                        handle.retry_states.insert(
                            node_id.clone(),
                            RetryState::Waiting {
                                attempt: failed_attempts,
                                next_at_ms: next_at,
                            },
                        );
                        self.event_bus.publish(WorkflowEvent::NodeRetryScheduled {
                            instance_id: handle.instance_id.clone(),
                            node_id: node_id.clone(),
                            attempt: failed_attempts,
                            next_at_ms: next_at,
                        })?;
                        self.persist_node_retry(handle, &spec, attempt, &error.to_string())?;
                    }
                    false => {
                        let next = transition_node_state(NodeState::Running, NodeTransition::Fail)?;
                        handle.node_states.insert(node_id.clone(), next);
                        handle
                            .retry_states
                            .insert(node_id.clone(), RetryState::Exhausted);
                        self.event_bus.publish(WorkflowEvent::NodeFailed {
                            instance_id: handle.instance_id.clone(),
                            node_id: node_id.clone(),
                            message: error.to_string(),
                        })?;
                        self.persist_node_fail(
                            handle,
                            &spec,
                            attempt,
                            duration_ms,
                            &error.to_string(),
                        )?;
                        self.apply_workflow_transition(handle, WorkflowTransition::BeginFail)?;
                        self.apply_workflow_transition(handle, WorkflowTransition::FinishFail)?;
                        self.event_bus.publish(WorkflowEvent::WorkflowFailed {
                            instance_id: handle.instance_id.clone(),
                            message: error.to_string(),
                        })?;
                        self.persist_instance(
                            handle,
                            Some("node_failed"),
                            Some(&error.to_string()),
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    fn apply_branch_skips(
        &self,
        handle: &mut SchedulerHandle,
        node_id: &NodeId,
        branch: Option<&str>,
    ) -> Result<(), WorkflowError> {
        let spec = match handle.graph.node(node_id) {
            Some(spec) => spec,
            None => return Ok(()),
        };
        match spec.node_type {
            NodeType::If | NodeType::Switch => {}
            _ => return Ok(()),
        }
        let chosen = match branch {
            Some(value) => value,
            None => return Ok(()),
        };
        let map = match handle.graph.branch_successors.get(node_id) {
            Some(map) => map,
            None => return Ok(()),
        };
        for (key, targets) in map {
            match key.as_str() == chosen || (key.is_empty() && chosen.is_empty()) {
                true => {}
                false => {
                    for target in targets {
                        let current = match handle.node_states.get(target) {
                            Some(state) => *state,
                            None => NodeState::Pending,
                        };
                        match current {
                            NodeState::Pending | NodeState::Blocked | NodeState::Ready => {
                                let next = transition_node_state(current, NodeTransition::Skip)?;
                                handle.node_states.insert(target.clone(), next);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn promote_retry_ready(&self, handle: &mut SchedulerHandle) -> Result<(), WorkflowError> {
        let now = now_ms();
        let mut updates = Vec::new();
        for (node_id, state) in &handle.node_states {
            match state {
                NodeState::RetryWaiting => match handle.retry_states.get(node_id) {
                    Some(RetryState::Waiting { next_at_ms, .. }) => match *next_at_ms <= now {
                        true => updates.push(node_id.clone()),
                        false => {}
                    },
                    _ => updates.push(node_id.clone()),
                },
                _ => {}
            }
        }
        for node_id in updates {
            let next = transition_node_state(NodeState::RetryWaiting, NodeTransition::MakeReady)?;
            handle.node_states.insert(node_id, next);
        }
        Ok(())
    }

    fn cancel_active_nodes(&self, handle: &mut SchedulerHandle) -> Result<(), WorkflowError> {
        let ids: Vec<NodeId> = handle.node_states.keys().cloned().collect();
        for node_id in ids {
            let current = match handle.node_states.get(&node_id) {
                Some(state) => *state,
                None => continue,
            };
            match current.is_terminal() {
                true => {}
                false => match transition_node_state(current, NodeTransition::Cancel) {
                    Ok(next) => {
                        handle.node_states.insert(node_id, next);
                    }
                    Err(_) => {}
                },
            }
        }
        Ok(())
    }

    fn finish_if_possible(&self, handle: &mut SchedulerHandle) -> Result<(), WorkflowError> {
        match handle.workflow_state {
            WorkflowState::Running => {}
            _ => return Ok(()),
        }

        let any_failed = handle
            .node_states
            .values()
            .any(|state| matches!(state, NodeState::Failed));
        match any_failed {
            true => return Ok(()),
            false => {}
        }

        let all_terminal = handle.node_states.values().all(|state| state.is_terminal());
        let any_running = handle.node_states.values().any(|state| {
            matches!(
                state,
                NodeState::Running
                    | NodeState::RetryWaiting
                    | NodeState::Ready
                    | NodeState::Pending
                    | NodeState::Blocked
            )
        });

        match all_terminal && !any_running {
            true => {
                // recompute: if all terminal
                let truly_all = handle.node_states.values().all(|s| s.is_terminal());
                match truly_all {
                    true => {
                        self.apply_workflow_transition(handle, WorkflowTransition::BeginComplete)?;
                        self.apply_workflow_transition(handle, WorkflowTransition::FinishComplete)?;
                        self.event_bus.publish(WorkflowEvent::WorkflowCompleted {
                            instance_id: handle.instance_id.clone(),
                        })?;
                        self.persist_instance(handle, None, None)?;
                    }
                    false => {}
                }
            }
            false => {
                // Start-only path: if every node that exists is terminal OR no ready and no running and all non-pending are done
                let has_active = handle.node_states.values().any(|state| {
                    matches!(
                        state,
                        NodeState::Running
                            | NodeState::RetryWaiting
                            | NodeState::Ready
                            | NodeState::Pending
                            | NodeState::Blocked
                    )
                });
                match has_active {
                    false => {
                        self.apply_workflow_transition(handle, WorkflowTransition::BeginComplete)?;
                        self.apply_workflow_transition(handle, WorkflowTransition::FinishComplete)?;
                        self.event_bus.publish(WorkflowEvent::WorkflowCompleted {
                            instance_id: handle.instance_id.clone(),
                        })?;
                        self.persist_instance(handle, None, None)?;
                    }
                    true => {}
                }
            }
        }
        Ok(())
    }

    fn apply_workflow_transition(
        &self,
        handle: &mut SchedulerHandle,
        transition: WorkflowTransition,
    ) -> Result<(), WorkflowError> {
        let next = transition_workflow_state(handle.workflow_state, transition)?;
        handle.workflow_state = next;
        Ok(())
    }

    fn persist_instance(
        &self,
        handle: &SchedulerHandle,
        error_code: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<(), WorkflowError> {
        let now = now_rfc3339();
        let finished = match handle.workflow_state.is_terminal() {
            true => Some(now.clone()),
            false => None,
        };
        let record = WfRtInstanceRecord {
            instance_id: handle.instance_id.as_str().to_string(),
            definition_id: Some(handle.definition.id.as_str().to_string()),
            definition_json: serde_json::to_string(&handle.definition).map_err(|error| {
                WorkflowError::persistence(format!("serialize definition: {error}"))
            })?,
            state: handle.workflow_state.as_str().to_string(),
            context_json: serde_json::to_string(&handle.context.to_value()).map_err(|error| {
                WorkflowError::persistence(format!("serialize context: {error}"))
            })?,
            context_version: handle.context.version() as i64,
            error_code: error_code.map(|value| value.to_string()),
            error_message: error_message.map(|value| value.to_string()),
            heartbeat_at: Some(now.clone()),
            created_at: now.clone(),
            updated_at: now,
            started_at: Some(now_rfc3339()),
            finished_at: finished,
        };
        self.checkpoint.update_instance(&record)
    }

    fn persist_node_success(
        &self,
        handle: &SchedulerHandle,
        spec: &crate::definition::NodeSpec,
        attempt: u32,
        duration_ms: i64,
        message: &Option<String>,
    ) -> Result<(), WorkflowError> {
        let now = now_rfc3339();
        let instance = self.instance_record(handle, None, None)?;
        let node = WfRtNodeRecord {
            instance_id: handle.instance_id.as_str().to_string(),
            node_id: spec.id.as_str().to_string(),
            node_type: spec.node_type.as_str().to_string(),
            state: NodeState::Succeeded.as_str().to_string(),
            attempt: attempt as i64,
            max_retry: spec.retry.max_retry as i64,
            retry_state_json: RetryState::NotRetrying.to_json_string(),
            input_json: Some(spec.config.to_string()),
            output_json: Some(json!({ "message": message }).to_string()),
            error_message: None,
            started_at: Some(now.clone()),
            finished_at: Some(now),
            duration_ms: Some(duration_ms),
        };
        self.checkpoint.commit_node_progress(
            &instance,
            &node,
            "node_completed",
            &json!({ "message": message }).to_string(),
        )
    }

    fn persist_node_fail(
        &self,
        handle: &SchedulerHandle,
        spec: &crate::definition::NodeSpec,
        attempt: u32,
        duration_ms: i64,
        message: &str,
    ) -> Result<(), WorkflowError> {
        let now = now_rfc3339();
        let instance = self.instance_record(handle, Some("node_failed"), Some(message))?;
        let node = WfRtNodeRecord {
            instance_id: handle.instance_id.as_str().to_string(),
            node_id: spec.id.as_str().to_string(),
            node_type: spec.node_type.as_str().to_string(),
            state: NodeState::Failed.as_str().to_string(),
            attempt: attempt as i64,
            max_retry: spec.retry.max_retry as i64,
            retry_state_json: RetryState::Exhausted.to_json_string(),
            input_json: Some(spec.config.to_string()),
            output_json: None,
            error_message: Some(message.to_string()),
            started_at: Some(now.clone()),
            finished_at: Some(now),
            duration_ms: Some(duration_ms),
        };
        self.checkpoint
            .commit_node_progress(&instance, &node, "node_failed", message)
    }

    fn persist_node_retry(
        &self,
        handle: &SchedulerHandle,
        spec: &crate::definition::NodeSpec,
        attempt: u32,
        message: &str,
    ) -> Result<(), WorkflowError> {
        let instance = self.instance_record(handle, None, None)?;
        let retry_json = match handle.retry_states.get(&spec.id) {
            Some(state) => state.to_json_string(),
            None => RetryState::NotRetrying.to_json_string(),
        };
        let node = WfRtNodeRecord {
            instance_id: handle.instance_id.as_str().to_string(),
            node_id: spec.id.as_str().to_string(),
            node_type: spec.node_type.as_str().to_string(),
            state: NodeState::RetryWaiting.as_str().to_string(),
            attempt: attempt as i64,
            max_retry: spec.retry.max_retry as i64,
            retry_state_json: retry_json,
            input_json: Some(spec.config.to_string()),
            output_json: None,
            error_message: Some(message.to_string()),
            started_at: Some(now_rfc3339()),
            finished_at: None,
            duration_ms: None,
        };
        self.checkpoint
            .commit_node_progress(&instance, &node, "node_retry", message)
    }

    fn instance_record(
        &self,
        handle: &SchedulerHandle,
        error_code: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<WfRtInstanceRecord, WorkflowError> {
        let now = now_rfc3339();
        Ok(WfRtInstanceRecord {
            instance_id: handle.instance_id.as_str().to_string(),
            definition_id: Some(handle.definition.id.as_str().to_string()),
            definition_json: serde_json::to_string(&handle.definition).map_err(|error| {
                WorkflowError::persistence(format!("serialize definition: {error}"))
            })?,
            state: handle.workflow_state.as_str().to_string(),
            context_json: serde_json::to_string(&handle.context.to_value()).map_err(|error| {
                WorkflowError::persistence(format!("serialize context: {error}"))
            })?,
            context_version: handle.context.version() as i64,
            error_code: error_code.map(|value| value.to_string()),
            error_message: error_message.map(|value| value.to_string()),
            heartbeat_at: Some(now.clone()),
            created_at: now.clone(),
            updated_at: now,
            started_at: Some(now_rfc3339()),
            finished_at: None,
        })
    }
}

/// 从检查点重建句柄。
///
/// @author Xiaoman
/// @created 2026-07-23
pub fn handle_from_checkpoint(
    gateway: &CheckpointGateway,
    instance_id: &InstanceId,
    graph: WorkflowGraph,
    definition: WorkflowDefinition,
) -> Result<SchedulerHandle, WorkflowError> {
    let record = match gateway.get_instance(instance_id)? {
        Some(record) => record,
        None => {
            return Err(WorkflowError::InstanceNotFound {
                instance_id: instance_id.to_string(),
            });
        }
    };
    let workflow_state = match WorkflowState::parse(&record.state) {
        Some(state) => state,
        None => {
            return Err(WorkflowError::persistence(format!(
                "unknown workflow state: {}",
                record.state
            )));
        }
    };
    let context = context_from_record(&record)?;
    let node_rows = gateway.list_nodes(instance_id)?;
    let mut node_states = HashMap::new();
    let mut attempts = HashMap::new();
    let mut retry_states = HashMap::new();
    for row in node_rows {
        let node_id = NodeId::new(row.node_id.clone());
        let state = match NodeState::parse(&row.state) {
            Some(NodeState::Running) => NodeState::Ready, // crash 中 Running → 可重入 Ready
            Some(state) => state,
            None => NodeState::Pending,
        };
        node_states.insert(node_id.clone(), state);
        attempts.insert(node_id.clone(), row.attempt.max(0) as u32);
        retry_states.insert(node_id, RetryState::from_json_str(&row.retry_state_json));
    }
    for node_id in graph.nodes.keys() {
        node_states
            .entry(node_id.clone())
            .or_insert(NodeState::Pending);
    }

    Ok(SchedulerHandle {
        instance_id: instance_id.clone(),
        definition,
        graph,
        workflow_state,
        node_states,
        attempts,
        retry_states,
        context,
        cancel_requested: false,
        pause_requested: false,
    })
}
