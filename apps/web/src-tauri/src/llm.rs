//! LLM 设置 RPC helper：get / save / test-connection + `stored_llm_client`。

use app_core::AppState;
use common::contracts::{
    RuntimeIpcLlmSettingsGetResponse, RuntimeIpcLlmSettingsSaveRequest,
    RuntimeIpcLlmTestConnectionRequest, RuntimeIpcLlmTestConnectionResponse,
};
use ports::llm_settings::LlmSettingsRecord;
use serde_json::{json, Value};

fn record_to_response(record: LlmSettingsRecord) -> Value {
    let configured = record.configured();
    json!(RuntimeIpcLlmSettingsGetResponse {
        provider: record.provider,
        base_url: record.base_url,
        model_id: record.model_id,
        configured,
        has_api_key: record.has_api_key,
        tools_enabled: record.tools_enabled,
        memory_enabled: record.memory_enabled,
        knowledge_enabled: record.knowledge_enabled,
    })
}

/// 读取 LLM 设置元数据（不含 api_key）。
pub async fn settings_get(app: &AppState) -> Result<Value, String> {
    let store = app.llm_settings_store.clone();
    let record = tokio::task::spawn_blocking(move || store.get())
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?;
    Ok(match record {
        Some(value) => record_to_response(value),
        None => json!(RuntimeIpcLlmSettingsGetResponse {
            provider: String::new(),
            base_url: None,
            model_id: String::new(),
            configured: false,
            has_api_key: false,
            tools_enabled: true,
            memory_enabled: true,
            knowledge_enabled: true,
        }),
    })
}

/// 保存 LLM 设置；密钥写入 keyring，响应不回传密钥。
pub async fn settings_save(
    app: &AppState,
    req: RuntimeIpcLlmSettingsSaveRequest,
) -> Result<Value, String> {
    let store = app.llm_settings_store.clone();
    let provider = req.provider;
    let base_url = req.base_url;
    let model_id = req.model_id;
    let api_key = req.api_key;
    let tools_enabled = req.tools_enabled;
    let memory_enabled = req.memory_enabled;
    let knowledge_enabled = req.knowledge_enabled;
    let record = tokio::task::spawn_blocking(move || {
        store.save(
            &provider,
            base_url.as_deref(),
            &model_id,
            Some(api_key.as_str()),
            tools_enabled,
            memory_enabled,
            knowledge_enabled,
        )
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())?;
    Ok(record_to_response(record))
}

/// 测试 LLM 连接。
pub async fn test_connection(
    app: &AppState,
    req: RuntimeIpcLlmTestConnectionRequest,
) -> Result<Value, String> {
    let provider = req.provider.trim().to_string();
    let model_id = req.model_id.trim().to_string();
    let base_url = req
        .base_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let draft_key = req.api_key.trim().to_string();

    if provider.is_empty() || model_id.is_empty() {
        return Ok(json!(RuntimeIpcLlmTestConnectionResponse {
            ok: false,
            error_code: Some("LLM_NOT_CONFIGURED".into()),
            message: "Provider and model_id are required".into(),
        }));
    }

    let store = app.llm_settings_store.clone();
    let resolved_key = if draft_key.is_empty() {
        tokio::task::spawn_blocking(move || store.resolve_api_key())
            .await
            .map_err(|error| error.to_string())?
            .map_err(|error| error.to_string())?
            .unwrap_or_default()
    } else {
        draft_key
    };

    let needs_key = provider != "openai_compatible" && provider != "ollama";
    if needs_key && resolved_key.is_empty() {
        return Ok(json!(RuntimeIpcLlmTestConnectionResponse {
            ok: false,
            error_code: Some("LLM_NOT_CONFIGURED".into()),
            message: "API key is not configured".into(),
        }));
    }

    let client = agent::llm::LlmClient::new(agent::llm::Config {
        provider,
        base_url,
        model_id,
        api_key: resolved_key,
    })
    .map_err(|error| error.to_string())?;

    match client.test_connection().await {
        Ok(()) => Ok(json!(RuntimeIpcLlmTestConnectionResponse {
            ok: true,
            error_code: None,
            message: "ok".into(),
        })),
        Err(error) => Ok(json!(RuntimeIpcLlmTestConnectionResponse {
            ok: false,
            error_code: Some("LLM_TEST_FAILED".into()),
            message: error.to_string(),
        })),
    }
}

/// 从 SQLite 与 keyring 构造 Rust LLM 客户端（与桌面 `stored_llm_client` 同构）。
pub async fn stored_llm_client(app: &AppState) -> Result<agent::llm::LlmClient, String> {
    let store = app.llm_settings_store.clone();
    let (record, api_key) = tokio::task::spawn_blocking(move || {
        let record = store
            .get()
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "LLM is not configured".to_string())?;
        let api_key = store
            .resolve_api_key()
            .map_err(|error| error.to_string())?
            .unwrap_or_default();
        Ok::<_, String>((record, api_key))
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())?;

    let provider = record.provider.trim().to_string();
    let model_id = record.model_id.trim().to_string();
    if provider.is_empty() || model_id.is_empty() {
        return Err("Provider and model_id are required".to_string());
    }
    if provider != "openai_compatible" && provider != "ollama" && api_key.trim().is_empty() {
        return Err("API key is not configured".to_string());
    }

    agent::llm::LlmClient::new(agent::llm::Config {
        provider,
        base_url: record.base_url,
        model_id,
        api_key,
    })
    .map_err(|error| error.to_string())
}
