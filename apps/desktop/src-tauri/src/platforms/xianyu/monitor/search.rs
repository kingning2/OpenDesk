//! 闲鱼 Sidecar 搜索封装 — 监控与手动搜索共用。

use business::account::AccountService;
use common::contracts::ChannelSidecarSearchRequest;
use common::DingDaResult;
use platform::ali1688::resolve_account_platform;
use platform::xianyu::cookies::parse_credential;
use platform::xianyu::stores::InMemoryAccountStore;
use serde_json::Value;

use crate::shared::state::AppState;

pub async fn search_offers(
    state: &AppState,
    account_store: &InMemoryAccountStore,
    owner_id: i64,
    account_id: &str,
    keyword: &str,
    max_results: i64,
    headed: bool,
) -> DingDaResult<Vec<Value>> {
    let service = AccountService::new(account_store);
    let account = service
        .list(owner_id)
        .map_err(common::DingDaError::wrap)?
        .into_iter()
        .find(|item| item.account_id == account_id)
        .ok_or_else(|| common::DingDaError::validation("账号不存在"))?;

    let platform = resolve_account_platform(&account.account_id, &account.platform);
    if platform != "xianyu" {
        return Err(common::DingDaError::validation("仅支持闲鱼账号"));
    }

    let cookie = account.cookie.trim();
    if cookie.is_empty() {
        return Err(common::DingDaError::validation(
            "账号 Cookie 为空，请先扫码登录",
        ));
    }
    let cookies = parse_credential(cookie);
    if cookies.is_empty() {
        return Err(common::DingDaError::validation("无法解析账号 Cookie"));
    }

    let sidecar_request = ChannelSidecarSearchRequest {
        account_id: account_id.to_string(),
        keyword: keyword.trim().to_string(),
        cookies,
        max_results: Some(max_results.clamp(1, 120)),
        headed: Some(headed),
        platform: Some("xianyu".to_string()),
        trace_id: Some(format!("xianyu-monitor-{account_id}-{keyword}")),
    };

    let response: Value = state
        .lifecycle
        .client()
        .post_json("/v1/channel/search", &sidecar_request)
        .await
        .map_err(common::DingDaError::wrap)?;

    let offers = response
        .get("offers")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    Ok(offers)
}
