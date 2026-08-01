//! Mail external integration settings Tauri IPC commands.
//!
//! 作者：coisini
//! 创建时间：2026-08-01

use mail::app::{GetEmailReadIntegration, ProbeEmailReadIntegration, SaveEmailReadIntegration};
use ports::mail::MailEmailReadIntegrationConfig;
use serde::{Deserialize, Serialize};

use crate::app::state::AppState;

/// IPC DTO for email-read integration settings (matches React contract).
///
/// 作者：coisini
/// 创建时间：2026-08-01
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MailEmailReadIntegrationConfigDto {
    /// Whether open-tracking integration is enabled.
    pub enabled: bool,
    /// API base URL (no trailing slash).
    pub api_base: String,
    /// Pixel path template with `{{email}}` / `{{mailId}}` placeholders.
    pub pixel_path_template: String,
    /// Query path template with `{{email}}` / `{{mailId}}` placeholders.
    pub query_path_template: String,
    /// JavaScript `parseResponse(data)` source for the settings UI test runner.
    pub parse_script: String,
}

impl From<MailEmailReadIntegrationConfig> for MailEmailReadIntegrationConfigDto {
    fn from(config: MailEmailReadIntegrationConfig) -> Self {
        Self {
            enabled: config.enabled,
            api_base: config.api_base,
            pixel_path_template: config.pixel_path_template,
            query_path_template: config.query_path_template,
            parse_script: config.parse_script,
        }
    }
}

impl From<MailEmailReadIntegrationConfigDto> for MailEmailReadIntegrationConfig {
    fn from(config: MailEmailReadIntegrationConfigDto) -> Self {
        Self {
            enabled: config.enabled,
            api_base: config.api_base,
            pixel_path_template: config.pixel_path_template,
            query_path_template: config.query_path_template,
            parse_script: config.parse_script,
        }
    }
}

/// Probe request for the settings UI test runner.
///
/// 作者：coisini
/// 创建时间：2026-08-01
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MailEmailReadIntegrationProbeRequest {
    /// Draft or saved integration config.
    pub config: MailEmailReadIntegrationConfigDto,
    /// Recipient email used in the query URL.
    pub recipient_email: String,
    /// Tracking id (`mailId`) used in the query URL.
    pub tracking_id: String,
}

/// Probe response with raw JSON from the external API.
///
/// 作者：coisini
/// 创建时间：2026-08-01
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct MailEmailReadIntegrationProbeResponse {
    /// Raw JSON body returned by the external query endpoint.
    pub response_json: String,
}

/// Load persisted email-read integration settings.
///
/// 作者：coisini
/// 创建时间：2026-08-01
#[tauri::command]
pub async fn mail_email_read_integration_get(
    state: tauri::State<'_, AppState>,
) -> Result<MailEmailReadIntegrationConfigDto, String> {
    let store = state.mail_store.clone();
    let config = tauri::async_runtime::spawn_blocking(move || {
        GetEmailReadIntegration::execute(store.as_ref())
    })
    .await
    .map_err(|error| error.to_string())??;
    Ok(config.into())
}

/// Save email-read integration settings from the settings UI.
///
/// 作者：coisini
/// 创建时间：2026-08-01
#[tauri::command]
pub async fn mail_email_read_integration_save(
    state: tauri::State<'_, AppState>,
    request: MailEmailReadIntegrationConfigDto,
) -> Result<MailEmailReadIntegrationConfigDto, String> {
    let store = state.mail_store.clone();
    let config = MailEmailReadIntegrationConfig::from(request);
    let saved = tauri::async_runtime::spawn_blocking(move || {
        SaveEmailReadIntegration::execute(store.as_ref(), config)
    })
    .await
    .map_err(|error| error.to_string())??;
    Ok(saved.into())
}

/// Probe query URL and return raw JSON for UI script testing.
///
/// 作者：coisini
/// 创建时间：2026-08-01
#[tauri::command]
pub async fn mail_email_read_integration_probe(
    request: MailEmailReadIntegrationProbeRequest,
) -> Result<MailEmailReadIntegrationProbeResponse, String> {
    let config = MailEmailReadIntegrationConfig::from(request.config);
    let recipient_email = request.recipient_email;
    let tracking_id = request.tracking_id;
    let body = tauri::async_runtime::spawn_blocking(move || {
        ProbeEmailReadIntegration::execute(&config, &recipient_email, &tracking_id)
    })
    .await
    .map_err(|error| error.to_string())??;
    Ok(MailEmailReadIntegrationProbeResponse {
        response_json: body,
    })
}
