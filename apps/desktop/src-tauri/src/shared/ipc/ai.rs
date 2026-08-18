//! AI ???? Tauri commands?
//!
//! ???coisini
//! ?????2026-08-11

use common::contracts::{AiIpcConfigRequest, AiIpcConfigResponse};
use common::DingDaResult;
use dingda_macros::timed;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::ai_config::AiConfigStore;
use crate::shared::ipc::IpcResponse;

/// API Key ????????
///
/// `ok` ??? Key ??;`message` ????????
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiApiKeyTestResult {
    pub ok: bool,
    pub message: String,
}

/// ?? AI ?? IPC?
///
/// ???coisini
/// ?????2026-08-11
///
/// # ??
/// - `state` ? AI ????
///
/// # ???
/// ?????????;??????????
#[tauri::command]
#[timed]
pub async fn ai_config_get(
    state: tauri::State<'_, Arc<AiConfigStore>>,
) -> DingDaResult<IpcResponse<AiIpcConfigResponse>> {
    let result = state.get().await?;
    Ok(IpcResponse::ok(result))
}

/// ?? AI ?? IPC(????)?
///
/// ???coisini
/// ?????2026-08-11
///
/// # ??
/// - `state` ? AI ????
/// - `config` ? ?????????
///
/// # ???
/// ????????
#[tauri::command]
#[timed]
pub async fn ai_config_set(
    state: tauri::State<'_, Arc<AiConfigStore>>,
    config: AiIpcConfigRequest,
) -> DingDaResult<IpcResponse<AiIpcConfigResponse>> {
    let result = state.set(config).await?;
    Ok(IpcResponse::ok(result))
}

/// ?? API Key ?????
///
/// ?????????????????????????????
/// ??????? DeepSeek ???`GET {base_url}/user/balance`?
///
/// ???coisini
/// ?????2026-08-11
///
/// # ??
/// - `base_url` ? ??????
/// - `api_key` ? ???? API Key
///
/// # ???
/// ????;`ok` ??????
#[tauri::command]
#[timed]
pub async fn ai_test_api_key(
    base_url: String,
    api_key: String,
) -> DingDaResult<IpcResponse<AiApiKeyTestResult>> {
    let url = format!("{}/user/balance", base_url.trim_end_matches('/'));
    let response = reqwest::Client::new()
        .get(url)
        .header("Authorization", format!("Bearer {}", api_key.trim()))
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|error| format!("????: {error}"))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()).into());
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|error| format!("????: {error}"))?;

    let has_price = body
        .get("balance_infos")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|infos| !infos.is_empty());

    Ok(IpcResponse::ok(AiApiKeyTestResult {
        ok: has_price,
        message: if has_price {
            "API Key ??".to_string()
        } else {
            "???????".to_string()
        },
    }))
}
