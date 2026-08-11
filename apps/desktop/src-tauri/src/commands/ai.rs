//! AI 配置相关 Tauri commands。
//!
//! 作者：coisini
//! 创建时间：2026-08-11

use common::contracts::{AiIpcConfigRequest, AiIpcConfigResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::ai_config::AiConfigStore;

/// API Key 可用性测试结果。
///
/// `ok` 表示该 Key 可用;`message` 为人类可读提示。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiApiKeyTestResult {
    pub ok: bool,
    pub message: String,
}

/// 读取 AI 配置 IPC。
///
/// 作者：coisini
/// 创建时间：2026-08-11
///
/// # 参数
/// - `state` — AI 配置存储
///
/// # 返回值
/// 当前平台与账号配置;首次启动返回空配置。
#[tauri::command]
pub async fn ai_config_get(
    state: tauri::State<'_, Arc<AiConfigStore>>,
) -> Result<AiIpcConfigResponse, String> {
    state.get().await
}

/// 写入 AI 配置 IPC(整体保存)。
///
/// 作者：coisini
/// 创建时间：2026-08-11
///
/// # 参数
/// - `state` — AI 配置存储
/// - `config` — 待持久化的完整配置
///
/// # 返回值
/// 持久化后的配置。
#[tauri::command]
pub async fn ai_config_set(
    state: tauri::State<'_, Arc<AiConfigStore>>,
    config: AiIpcConfigRequest,
) -> Result<AiIpcConfigResponse, String> {
    state.set(config).await
}

/// 测试 API Key 是否可用。
///
/// 通过平台余额接口校验：只要返回余额信息（价格）即视为可用。
/// 目前余额接口仅 DeepSeek 提供：`GET {base_url}/user/balance`。
///
/// 作者：coisini
/// 创建时间：2026-08-11
///
/// # 参数
/// - `base_url` — 平台接口地址
/// - `api_key` — 待校验的 API Key
///
/// # 返回值
/// 校验结果;`ok` 为是否可用。
#[tauri::command]
pub async fn ai_test_api_key(
    base_url: String,
    api_key: String,
) -> Result<AiApiKeyTestResult, String> {
    let url = format!("{}/user/balance", base_url.trim_end_matches('/'));
    let response = reqwest::Client::new()
        .get(url)
        .header("Authorization", format!("Bearer {}", api_key.trim()))
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|error| format!("请求失败: {error}"))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|error| format!("解析失败: {error}"))?;

    let has_price = body
        .get("balance_infos")
        .and_then(serde_json::Value::as_array)
        .map_or(false, |infos| !infos.is_empty());

    Ok(AiApiKeyTestResult {
        ok: has_price,
        message: if has_price {
            "API Key 有效".to_string()
        } else {
            "未返回余额信息".to_string()
        },
    })
}
