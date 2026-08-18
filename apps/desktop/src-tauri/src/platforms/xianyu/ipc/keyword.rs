//! 自动回复关键词 Tauri commands — 关键词 CRUD 与整表替换。

use crate::platforms::xianyu::persist::InMemoryKeywordStore;
use crate::shared::ipc::IpcResponse;
use app::auto_reply::{KeywordRule, KeywordService};
use common;
use serde::Deserialize;
use std::sync::Arc;
use tauri::State;

/// 关键词服务句柄（setup 时注册到 Tauri 状态）。
pub struct KeywordHandle {
    pub store: Arc<InMemoryKeywordStore>,
}

/// 关键词列表请求。
#[derive(Debug, Deserialize)]
pub struct KeywordListRequest {
    pub account_id: String,
}

/// 整表替换保存请求。
#[derive(Debug, Deserialize)]
pub struct KeywordReplaceRequest {
    pub account_id: String,
    pub keywords: Vec<KeywordRule>,
}

/// 新增关键词请求。
#[derive(Debug, Deserialize)]
pub struct KeywordAddRequest {
    pub account_id: String,
    pub rule: KeywordRule,
}

/// 删除关键词请求。
#[derive(Debug, Deserialize)]
pub struct KeywordDeleteRequest {
    pub rule_id: i64,
}

/// 查询账号关键词。
#[tauri::command]
pub fn keyword_list(
    state: State<'_, KeywordHandle>,
    request: KeywordListRequest,
) -> common::OpenDeskResult<IpcResponse<Vec<KeywordRule>>> {
    let service = KeywordService::new(state.store.as_ref());
    let result = service.list(&request.account_id)?;
    Ok(IpcResponse::ok(result))
}

/// 整表替换保存。
#[tauri::command]
pub fn keyword_replace(
    state: State<'_, KeywordHandle>,
    request: KeywordReplaceRequest,
) -> common::OpenDeskResult<IpcResponse<()>> {
    let service = KeywordService::new(state.store.as_ref());
    service.replace(&request.account_id, &request.keywords)?;
    Ok(IpcResponse::ok(()))
}

/// 新增关键词。
#[tauri::command]
pub fn keyword_add(
    state: State<'_, KeywordHandle>,
    request: KeywordAddRequest,
) -> common::OpenDeskResult<IpcResponse<KeywordRule>> {
    let service = KeywordService::new(state.store.as_ref());
    let result = service.add(&request.account_id, request.rule)?;
    Ok(IpcResponse::ok(result))
}

/// 删除关键词。
#[tauri::command]
pub fn keyword_delete(
    state: State<'_, KeywordHandle>,
    request: KeywordDeleteRequest,
) -> common::OpenDeskResult<IpcResponse<()>> {
    let service = KeywordService::new(state.store.as_ref());
    service.delete(request.rule_id)?;
    Ok(IpcResponse::ok(()))
}
