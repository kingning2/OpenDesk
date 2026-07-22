//! LLM Provider 设置 Tauri commands。
//!
//! 约束：get/save 响应**永不**回传 api_key（含脱敏）。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-22

use std::collections::HashMap;

use common::contracts::{
    RuntimeIpcLlmSettingsGetResponse, RuntimeIpcLlmSettingsSaveRequest,
    RuntimeIpcLlmSettingsSaveResponse, RuntimeIpcLlmTestConnectionRequest,
    RuntimeIpcLlmTestConnectionResponse, RuntimeSidecarLlmTestConnectionRequest,
};
use ports::llm_settings::LlmSettingsRecord;
use ports::sidecar::AgentSidecarGateway;
use uuid::Uuid;

use crate::state::AppState;

fn record_to_get_response(record: LlmSettingsRecord) -> RuntimeIpcLlmSettingsGetResponse {
    let configured = record.configured();
    RuntimeIpcLlmSettingsGetResponse {
        provider: record.provider,
        base_url: record.base_url,
        model_id: record.model_id,
        configured,
        has_api_key: record.has_api_key,
    }
}

fn record_to_save_response(record: LlmSettingsRecord) -> RuntimeIpcLlmSettingsSaveResponse {
    let configured = record.configured();
    RuntimeIpcLlmSettingsSaveResponse {
        provider: record.provider,
        base_url: record.base_url,
        model_id: record.model_id,
        configured,
        has_api_key: record.has_api_key,
    }
}

/// 根据已存设置构造 Sidecar 环境变量（含内存态 API Key）。
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
///
/// # 参数
/// - `store` — LLM 设置存储
///
/// # 返回值
/// `OPENDESK_LLM_*` 环境表。
fn build_llm_process_env(
    store: &dyn ports::llm_settings::LlmSettingsStore,
) -> Result<HashMap<String, String>, String> {
    let mut env = HashMap::new();
    let Some(record) = store.get().map_err(|error| error.to_string())? else {
        return Ok(env);
    };
    env.insert("OPENDESK_LLM_PROVIDER".into(), record.provider);
    if let Some(base_url) = record.base_url {
        env.insert("OPENDESK_LLM_BASE_URL".into(), base_url);
    }
    env.insert("OPENDESK_LLM_MODEL_ID".into(), record.model_id);
    if let Some(api_key) = store.resolve_api_key().map_err(|error| error.to_string())? {
        env.insert("OPENDESK_LLM_API_KEY".into(), api_key);
    }
    Ok(env)
}

/// 读取 LLM 设置元数据（不含 api_key）。
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
///
/// # 参数
/// - `state` — 应用共享状态
///
/// # 返回值
/// 当前配置；从未保存时字段为空且 `configured=false`。
#[tauri::command]
pub async fn llm_settings_get(
    state: tauri::State<'_, AppState>,
) -> Result<RuntimeIpcLlmSettingsGetResponse, String> {
    let store = state.llm_settings_store.clone();
    let record = tauri::async_runtime::spawn_blocking(move || store.get())
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?;

    Ok(match record {
        Some(value) => record_to_get_response(value),
        None => RuntimeIpcLlmSettingsGetResponse {
            provider: String::new(),
            base_url: None,
            model_id: String::new(),
            configured: false,
            has_api_key: false,
        },
    })
}

/// 保存 LLM 设置；密钥写入 OS keyring，响应不回传密钥。
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
///
/// # 参数
/// - `state` — 应用共享状态
/// - `request` — 保存请求；`api_key` 空串表示保留原密钥
///
/// # 返回值
/// 保存后的元数据（无 api_key）。
#[tauri::command]
pub async fn llm_settings_save(
    state: tauri::State<'_, AppState>,
    request: RuntimeIpcLlmSettingsSaveRequest,
) -> Result<RuntimeIpcLlmSettingsSaveResponse, String> {
    let store = state.llm_settings_store.clone();
    let provider = request.provider;
    let base_url = request.base_url;
    let model_id = request.model_id;
    let api_key = request.api_key;
    let record = tauri::async_runtime::spawn_blocking(move || {
        store.save(
            &provider,
            base_url.as_deref(),
            &model_id,
            Some(api_key.as_str()),
        )
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())?;

    let env = build_llm_process_env(state.llm_settings_store.as_ref())?;
    state.lifecycle.set_process_env(env).await;
    // 保存后重载 Sidecar，使注入环境生效。
    if let Err(error) = state.lifecycle.restart().await {
        tracing::warn!(%error, "sidecar restart after llm_settings_save failed");
    }

    Ok(record_to_save_response(record))
}

/// 测试 LLM 连接（草稿或已存密钥）；响应不含 api_key。
///
/// 作者：Xiaoman
/// 创建时间：2026-07-22
///
/// # 参数
/// - `state` — 应用共享状态
/// - `request` — 探测请求；`api_key` 空则从 keyring 解析
///
/// # 返回值
/// 探测结果；未配置时 `error_code=LLM_NOT_CONFIGURED`。
#[tauri::command]
pub async fn llm_test_connection(
    state: tauri::State<'_, AppState>,
    request: RuntimeIpcLlmTestConnectionRequest,
) -> Result<RuntimeIpcLlmTestConnectionResponse, String> {
    let provider = request.provider.trim().to_string();
    let model_id = request.model_id.trim().to_string();
    let base_url = request
        .base_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let draft_key = request.api_key.trim().to_string();

    if provider.is_empty() || model_id.is_empty() {
        return Ok(RuntimeIpcLlmTestConnectionResponse {
            ok: false,
            error_code: Some("LLM_NOT_CONFIGURED".into()),
            message: "Provider and model_id are required".into(),
        });
    }

    let store = state.llm_settings_store.clone();
    let resolved_key = if draft_key.is_empty() {
        tauri::async_runtime::spawn_blocking(move || store.resolve_api_key())
            .await
            .map_err(|error| error.to_string())?
            .map_err(|error| error.to_string())?
            .unwrap_or_default()
    } else {
        draft_key
    };

    let needs_key = provider != "openai_compatible";
    if needs_key && resolved_key.is_empty() {
        return Ok(RuntimeIpcLlmTestConnectionResponse {
            ok: false,
            error_code: Some("LLM_NOT_CONFIGURED".into()),
            message: "API key is not configured".into(),
        });
    }

    state
        .lifecycle
        .ensure_running()
        .await
        .map_err(|error| error.to_string())?;

    let sidecar_request = RuntimeSidecarLlmTestConnectionRequest {
        provider,
        base_url,
        model_id,
        api_key: resolved_key,
        trace_id: Some(Uuid::new_v4().to_string()),
    };

    let sidecar_response = state
        .gateway
        .llm_test_connection(sidecar_request)
        .await
        .map_err(|error| error.to_string())?;

    Ok(RuntimeIpcLlmTestConnectionResponse {
        ok: sidecar_response.ok,
        error_code: sidecar_response.error_code.or_else(|| {
            if sidecar_response.ok {
                None
            } else {
                Some("LLM_TEST_FAILED".into())
            }
        }),
        message: sidecar_response.message,
    })
}
