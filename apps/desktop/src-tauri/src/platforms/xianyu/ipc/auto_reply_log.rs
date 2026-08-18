//! 自动回复日志 Tauri commands。

use crate::platforms::xianyu::persist::InMemoryAutoReplyLogStore;
use crate::shared::ipc::IpcResponse;
use app::auto_reply::{AutoReplyLogPage, AutoReplyLogQuery, AutoReplyLogService};
use common;
use serde::Deserialize;
use std::sync::Arc;
use tauri::State;

/// 日志服务句柄（setup 时注册到 Tauri 状态）。
pub struct AutoReplyLogHandle {
    pub store: Arc<InMemoryAutoReplyLogStore>,
}

/// 日志查询请求。
#[derive(Debug, Deserialize)]
pub struct LogListRequest {
    pub owner_id: i64,
    pub page: u32,
    pub page_size: u32,
    #[serde(default)]
    pub account_id: String,
    #[serde(default)]
    pub start_date: String,
    #[serde(default)]
    pub end_date: String,
    #[serde(default)]
    pub matched_rule_type: String,
    #[serde(default)]
    pub send_status: String,
    #[serde(default)]
    pub message_type: String,
}

/// 分页查询自动回复日志。
#[tauri::command]
pub fn auto_reply_log_list(
    state: State<'_, AutoReplyLogHandle>,
    request: LogListRequest,
) -> common::OpenDeskResult<IpcResponse<AutoReplyLogPage>> {
    let service = AutoReplyLogService::new(state.store.as_ref());
    let query = AutoReplyLogQuery {
        page: request.page,
        page_size: request.page_size,
        account_id: request.account_id,
        start_date: request.start_date,
        end_date: request.end_date,
        matched_rule_type: request.matched_rule_type,
        send_status: request.send_status,
        message_type: request.message_type,
    };
    let result = service.list(request.owner_id, &query)?;
    Ok(IpcResponse::ok(result))
}
