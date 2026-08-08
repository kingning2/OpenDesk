//! Workflow Runtime Tauri IPC（start / cancel / resume / active）。
//!
//! 作者：coisini
//! 创建时间：2026-07-23

use common::contracts::{
    WorkflowRuntimeIpcActiveRequest, WorkflowRuntimeIpcActiveResponse,
    WorkflowRuntimeIpcCancelRequest, WorkflowRuntimeIpcCancelResponse,
    WorkflowRuntimeIpcResumeRequest, WorkflowRuntimeIpcResumeResponse,
    WorkflowRuntimeIpcStartRequest, WorkflowRuntimeIpcStartResponse,
};
use serde_json::json;
use workflow_runtime::{InstanceId, WorkflowContext, WorkflowDefinition};

use crate::app::state::AppState;

fn parse_context(context_json: Option<String>) -> Result<WorkflowContext, String> {
    let Some(raw) = context_json else {
        return Ok(WorkflowContext::new());
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(WorkflowContext::new());
    }
    serde_json::from_str(trimmed).map_err(|error| error.to_string())
}

/// 启动工作流实例（后台 detached 执行）。
#[tauri::command]
pub async fn workflow_runtime_start(
    state: tauri::State<'_, AppState>,
    request: WorkflowRuntimeIpcStartRequest,
) -> Result<WorkflowRuntimeIpcStartResponse, String> {
    let definition: WorkflowDefinition =
        serde_json::from_str(request.definition_json.trim()).map_err(|error| error.to_string())?;
    let context = parse_context(request.context_json)?;
    let facade = state.workflow_runtime.clone();

    match facade.start_detached(definition, context).await {
        Ok(instance_id) => Ok(WorkflowRuntimeIpcStartResponse {
            instance_id: instance_id.as_str().to_string(),
            state: "running".to_string(),
            error: None,
        }),
        Err(error) => Ok(WorkflowRuntimeIpcStartResponse {
            instance_id: String::new(),
            state: "failed".to_string(),
            error: Some(error.to_string()),
        }),
    }
}

/// 取消运行中的实例。
#[tauri::command]
pub async fn workflow_runtime_cancel(
    state: tauri::State<'_, AppState>,
    request: WorkflowRuntimeIpcCancelRequest,
) -> Result<WorkflowRuntimeIpcCancelResponse, String> {
    let instance_id = InstanceId::new(request.instance_id);
    let facade = state.workflow_runtime.clone();

    match facade.cancel(&instance_id).await {
        Ok(()) => Ok(WorkflowRuntimeIpcCancelResponse {
            instance_id: instance_id.as_str().to_string(),
            ok: true,
            error: None,
        }),
        Err(error) => Ok(WorkflowRuntimeIpcCancelResponse {
            instance_id: instance_id.as_str().to_string(),
            ok: false,
            error: Some(error.to_string()),
        }),
    }
}

/// 恢复可恢复实例。
#[tauri::command]
pub async fn workflow_runtime_resume(
    state: tauri::State<'_, AppState>,
    request: WorkflowRuntimeIpcResumeRequest,
) -> Result<WorkflowRuntimeIpcResumeResponse, String> {
    let instance_id = InstanceId::new(request.instance_id);
    let facade = state.workflow_runtime.clone();

    match facade.resume(&instance_id).await {
        Ok(state) => Ok(WorkflowRuntimeIpcResumeResponse {
            instance_id: instance_id.as_str().to_string(),
            state: state.as_str().to_string(),
            error: None,
        }),
        Err(error) => Ok(WorkflowRuntimeIpcResumeResponse {
            instance_id: instance_id.as_str().to_string(),
            state: "failed".to_string(),
            error: Some(error.to_string()),
        }),
    }
}

/// 查询可恢复实例列表。
#[tauri::command]
pub async fn workflow_runtime_active(
    state: tauri::State<'_, AppState>,
    _request: WorkflowRuntimeIpcActiveRequest,
) -> Result<WorkflowRuntimeIpcActiveResponse, String> {
    let records = state
        .workflow_runtime
        .list_recoverable()
        .map_err(|error| error.to_string())?;

    let items: Vec<serde_json::Value> = records
        .into_iter()
        .map(|record| {
            json!({
                "instance_id": record.instance_id,
                "state": record.state,
                "definition_id": record.definition_id,
                "updated_at": record.updated_at,
                "heartbeat_at": record.heartbeat_at,
                "error_message": record.error_message,
            })
        })
        .collect();

    let instances_json = serde_json::to_string(&items).map_err(|error| error.to_string())?;

    Ok(WorkflowRuntimeIpcActiveResponse { instances_json })
}
