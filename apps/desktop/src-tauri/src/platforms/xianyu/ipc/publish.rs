//! 单品发布 Tauri commands — 账号能力检测 + 发布执行。
//!
//! 壳层组合：`InMemoryPublishGateway` → `app::publish::PublishService`。

use crate::platforms::xianyu::adapter::InMemoryPublishGateway;
use app::publish::gateway::AccountCapability;
use app::publish::{PublishGateway, PublishRequest, PublishService, PublishServiceResult};
use common;
use serde::Deserialize;
use std::sync::Arc;
use tauri::State;

/// 发布服务句柄（setup 时注册到 Tauri 状态）。
pub struct PublishHandle {
    pub gateway: Arc<InMemoryPublishGateway>,
}

#[derive(Debug, Deserialize)]
pub struct CapabilityRequest {
    pub user_id: i64,
    pub account_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SinglePublishRequest {
    pub user_id: i64,
    pub account_id: String,
    pub item: serde_json::Value,
    #[serde(default)]
    pub material_id: Option<i64>,
}

/// 账号发布能力检测。
#[tauri::command]
pub async fn publish_capability(
    state: State<'_, PublishHandle>,
    request: CapabilityRequest,
) -> common::OpenDeskResult<AccountCapability> {
    let cookie = state
        .gateway
        .account_cookie(request.user_id, &request.account_id)?;
    let Some(cookie) = cookie else {
        return Ok(AccountCapability {
            success: false,
            is_fish_shop: false,
            cookies_str: None,
            message: "账号不存在或缺少 Cookie，无法发布".to_string(),
        });
    };
    state
        .gateway
        .detect_capability(&request.account_id, &cookie, request.user_id)
        .await
}

/// 执行单品发布。
#[tauri::command]
pub async fn publish_single(
    state: State<'_, PublishHandle>,
    request: SinglePublishRequest,
) -> common::OpenDeskResult<PublishServiceResult> {
    let service = PublishService::new(state.gateway.as_ref());
    let publish_request = PublishRequest {
        user_id: request.user_id,
        account_id: request.account_id,
        item: request.item,
        material_id: request.material_id,
    };
    Ok(service.execute(&publish_request).await)
}
