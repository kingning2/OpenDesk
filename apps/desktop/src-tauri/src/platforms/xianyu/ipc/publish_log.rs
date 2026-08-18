//! 发布日志 Tauri commands — 分页查询与按天数清理。
//!
//! 壳层组合：`InMemoryPublishLogStore` → `app::publish::PublishLogService`。

use crate::platforms::xianyu::persist::InMemoryPublishLogStore;
use app::publish::{PublishLog, PublishLogQuery, PublishLogService};
use common;
use serde::Deserialize;
use std::sync::Arc;
use tauri::State;

/// 日志服务句柄（setup 时注册到 Tauri 状态）。
pub struct PublishLogHandle {
    pub store: Arc<InMemoryPublishLogStore>,
}

/// 日志查询请求。
#[derive(Debug, Deserialize)]
pub struct PublishLogListRequest {
    pub owner_id: i64,
    pub page: u32,
    pub page_size: u32,
    #[serde(default)]
    pub account_id: String,
    #[serde(default)]
    pub status: String,
}

/// 清空日志请求。
#[derive(Debug, Deserialize)]
pub struct PublishLogClearRequest {
    pub owner_id: i64,
    /// 保留最近 N 天；0 表示清空全部。
    #[serde(default)]
    pub days: u32,
}

/// 分页查询发布日志。
#[tauri::command]
pub fn publish_log_list(
    state: State<'_, PublishLogHandle>,
    request: PublishLogListRequest,
) -> common::OpenDeskResult<(Vec<PublishLog>, u32)> {
    let service = PublishLogService::new(state.store.as_ref());
    let query = PublishLogQuery {
        page: request.page,
        page_size: request.page_size,
        account_id: request.account_id,
        status: request.status,
    };
    service.list(request.owner_id, &query)
}

/// 清空早于指定天数的发布日志。
#[tauri::command]
pub fn publish_log_clear(
    state: State<'_, PublishLogHandle>,
    request: PublishLogClearRequest,
) -> common::OpenDeskResult<()> {
    let service = PublishLogService::new(state.store.as_ref());
    service.clear_older_than(request.owner_id, request.days)
}
