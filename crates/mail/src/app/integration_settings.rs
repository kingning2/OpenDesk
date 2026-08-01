//! Email-read integration settings use cases (UI-configured external API).
//!
//! 作者：coisini
//! 创建时间：2026-08-01

use ports::mail::{MailEmailReadIntegrationConfig, MailStore};

use super::tracking::probe_open_status;

/// Load email-read integration settings for the settings UI.
///
/// 作者：coisini
/// 创建时间：2026-08-01
pub struct GetEmailReadIntegration;

impl GetEmailReadIntegration {
    /// Read persisted integration config.
    pub fn execute<S: MailStore + ?Sized>(
        store: &S,
    ) -> Result<MailEmailReadIntegrationConfig, String> {
        store
            .get_email_read_integration()
            .map_err(|error| error.to_string())
    }
}

/// Save email-read integration settings from the settings UI.
///
/// 作者：coisini
/// 创建时间：2026-08-01
pub struct SaveEmailReadIntegration;

impl SaveEmailReadIntegration {
    /// Persist integration config.
    pub fn execute<S: MailStore + ?Sized>(
        store: &S,
        config: MailEmailReadIntegrationConfig,
    ) -> Result<MailEmailReadIntegrationConfig, String> {
        store
            .save_email_read_integration(config)
            .map_err(|error| error.to_string())
    }
}

/// Probe query URL and return raw JSON for the UI script test runner.
///
/// 作者：coisini
/// 创建时间：2026-08-01
pub struct ProbeEmailReadIntegration;

impl ProbeEmailReadIntegration {
    /// Fetch open-status JSON using saved or draft config.
    pub fn execute(
        config: &MailEmailReadIntegrationConfig,
        recipient_email: &str,
        tracking_id: &str,
    ) -> Result<String, String> {
        probe_open_status(config, recipient_email, tracking_id)
    }
}
