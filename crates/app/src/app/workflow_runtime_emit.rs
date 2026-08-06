//! Bridge [`workflow_runtime::WorkflowEvent`] to Tauri `workflow_runtime:phase` events.
//!
//! 作者：coisini
//! 创建时间：2026-07-23

use common::contracts::WorkflowRuntimeEventPhase;
use tauri::{AppHandle, Emitter};
use workflow_runtime::{NodeState, WorkflowEvent, WorkflowState};

/// Runtime → UI 事件 topic（与前端 `WorkflowRuntimeUiEvent.Phase` 对齐）。
pub const WORKFLOW_RUNTIME_PHASE_TOPIC: &str = "workflow_runtime:phase";

/// 将 Workflow Runtime 领域事件推送到 React webview。
pub struct TauriWorkflowRuntimeEmitter {
    app: AppHandle,
}

impl TauriWorkflowRuntimeEmitter {
    /// 绑定运行中的 Tauri app。
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    /// 发布 phase 事件。
    pub fn emit_phase(&self, event: &WorkflowEvent) {
        let payload = event_to_phase(event);
        if let Err(error) = self.app.emit(WORKFLOW_RUNTIME_PHASE_TOPIC, &payload) {
            tracing::warn!(%error, "failed to emit workflow runtime phase");
        }
    }
}

fn event_to_phase(event: &WorkflowEvent) -> WorkflowRuntimeEventPhase {
    match event {
        WorkflowEvent::WorkflowStarted { instance_id, state } => WorkflowRuntimeEventPhase {
            kind: event.name().to_string(),
            instance_id: instance_id.as_str().to_string(),
            node_id: None,
            state: Some(state.as_str().to_string()),
            message: None,
            context_version: None,
        },
        WorkflowEvent::WorkflowCompleted { instance_id } => WorkflowRuntimeEventPhase {
            kind: event.name().to_string(),
            instance_id: instance_id.as_str().to_string(),
            node_id: None,
            state: Some(WorkflowState::Completed.as_str().to_string()),
            message: None,
            context_version: None,
        },
        WorkflowEvent::WorkflowFailed {
            instance_id,
            message,
        } => WorkflowRuntimeEventPhase {
            kind: event.name().to_string(),
            instance_id: instance_id.as_str().to_string(),
            node_id: None,
            state: Some(WorkflowState::Failed.as_str().to_string()),
            message: Some(message.clone()),
            context_version: None,
        },
        WorkflowEvent::WorkflowPaused { instance_id } => WorkflowRuntimeEventPhase {
            kind: event.name().to_string(),
            instance_id: instance_id.as_str().to_string(),
            node_id: None,
            state: Some(WorkflowState::Paused.as_str().to_string()),
            message: None,
            context_version: None,
        },
        WorkflowEvent::WorkflowCancelled { instance_id } => WorkflowRuntimeEventPhase {
            kind: event.name().to_string(),
            instance_id: instance_id.as_str().to_string(),
            node_id: None,
            state: Some(WorkflowState::Cancelled.as_str().to_string()),
            message: None,
            context_version: None,
        },
        WorkflowEvent::NodeStarted {
            instance_id,
            node_id,
        } => WorkflowRuntimeEventPhase {
            kind: event.name().to_string(),
            instance_id: instance_id.as_str().to_string(),
            node_id: Some(node_id.as_str().to_string()),
            state: Some(NodeState::Running.as_str().to_string()),
            message: None,
            context_version: None,
        },
        WorkflowEvent::NodeCompleted {
            instance_id,
            node_id,
            state,
        } => WorkflowRuntimeEventPhase {
            kind: event.name().to_string(),
            instance_id: instance_id.as_str().to_string(),
            node_id: Some(node_id.as_str().to_string()),
            state: Some(state.as_str().to_string()),
            message: None,
            context_version: None,
        },
        WorkflowEvent::NodeFailed {
            instance_id,
            node_id,
            message,
        } => WorkflowRuntimeEventPhase {
            kind: event.name().to_string(),
            instance_id: instance_id.as_str().to_string(),
            node_id: Some(node_id.as_str().to_string()),
            state: Some(NodeState::Failed.as_str().to_string()),
            message: Some(message.clone()),
            context_version: None,
        },
        WorkflowEvent::NodeRetryScheduled {
            instance_id,
            node_id,
            ..
        } => WorkflowRuntimeEventPhase {
            kind: event.name().to_string(),
            instance_id: instance_id.as_str().to_string(),
            node_id: Some(node_id.as_str().to_string()),
            state: Some(NodeState::RetryWaiting.as_str().to_string()),
            message: None,
            context_version: None,
        },
        WorkflowEvent::ContextChanged {
            instance_id,
            version,
        } => WorkflowRuntimeEventPhase {
            kind: event.name().to_string(),
            instance_id: instance_id.as_str().to_string(),
            node_id: None,
            state: None,
            message: None,
            context_version: Some(*version as i64),
        },
    }
}
