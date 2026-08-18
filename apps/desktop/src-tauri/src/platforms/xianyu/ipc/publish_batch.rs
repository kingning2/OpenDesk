//! 批量发布 Tauri commands。

use crate::platforms::xianyu::adapter::InMemoryPublishGateway;
use crate::platforms::xianyu::persist::InMemoryBatchStore;
use crate::shared::ipc::IpcResponse;
use app::publish::{BatchService, BatchStore, BatchTask, PublishRequest, PublishService};
use common;
use serde::Deserialize;
use std::sync::Arc;
use tauri::State;

pub struct BatchPublishHandle {
    pub store: Arc<InMemoryBatchStore>,
    pub gateway: Arc<InMemoryPublishGateway>,
}

#[derive(Debug, Deserialize)]
pub struct BatchSubmitRequest {
    pub owner_id: i64,
    pub account_ids: Vec<String>,
    pub material_ids: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct BatchStatusRequest {
    pub owner_id: i64,
    pub batch_id: String,
}

#[tauri::command]
pub async fn publish_batch_submit(
    state: State<'_, BatchPublishHandle>,
    request: BatchSubmitRequest,
) -> common::DingDaResult<IpcResponse<BatchTask>> {
    let batch_id = format!("batch-{}", uuid_fragment());
    let service = BatchService::new(state.store.as_ref());
    let task = service.submit(
        &batch_id,
        request.owner_id,
        &request.account_ids,
        &request.material_ids,
    )?;

    let store = state.store.clone();
    let gateway = state.gateway.clone();
    let task_snapshot = task.clone();
    let account_ids = task.account_ids.clone();
    let material_ids = task.material_ids.clone();
    tauri::async_runtime::spawn(async move {
        let publish = PublishService::new(gateway.as_ref());
        let mut task = task_snapshot;
        for account_id in &account_ids {
            for material_id in &material_ids {
                let item = serde_json::json!({
                    "title": format!("批量发布素材 #{material_id}"),
                    "description": "",
                    "price": 0,
                    "images": []
                });
                let result = publish
                    .execute(&PublishRequest {
                        user_id: task.owner_id,
                        account_id: account_id.clone(),
                        item,
                        material_id: Some(*material_id),
                    })
                    .await;
                task.record(account_id, result.success);
                task.mark_sync(
                    account_id,
                    "skipped",
                    "内存网关：批量发布商品同步由 sidecar 执行",
                    0,
                    0,
                );
                let _ = store.update_task(&task);
            }
        }
        if !task.finished {
            task.finished = true;
        }
        let _ = store.update_task(&task);
    });

    Ok(IpcResponse::ok(task))
}

#[tauri::command]
pub fn publish_batch_status(
    state: State<'_, BatchPublishHandle>,
    request: BatchStatusRequest,
) -> common::DingDaResult<IpcResponse<Option<BatchTask>>> {
    let service = BatchService::new(state.store.as_ref());
    let result = service.status(request.owner_id, &request.batch_id)?;
    Ok(IpcResponse::ok(result))
}

fn uuid_fragment() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    format!("{:x}", millis % 1_000_000_000)
}
