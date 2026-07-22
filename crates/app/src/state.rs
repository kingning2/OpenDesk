//! 应用运行时状态组装。
//!
//! 作者：coisini
//! 创建时间：2026-07-16

use adapter::agent_sidecar::RuntimeAgentSidecar;
#[cfg(not(feature = "license-lock"))]
use adapter::license::UnlockedLicenseGate;
#[cfg(feature = "license-lock")]
use adapter::license::{FailClosedLicenseGate, VerifierProcessLicense};
use crawler::CrawlerService;
use kernel::event::InMemoryEventBus;
use ports::crawler_channels::CrawlerChannelStore;
use ports::crawler_keywords::CrawlerKeywordStore;
use ports::crawler_settings::CrawlerSettingsStore;
use ports::customer::CustomerStore;
use ports::license::LicenseGate;
use ports::llm_settings::LlmSettingsStore;
use ports::mail::MailStore;
use ports::workflow::ScriptSnippetStore;
use runtime::sidecar::lifecycle::SidecarLifecycle;
use std::sync::Arc;

/// 桌面应用共享状态。
///
/// 功能：
///
/// - 持有 sidecar 生命周期与 Agent 网关
/// - 持有 License 闸门实现
/// - 持有进程内 crawler 与 SQLite stores
/// - 持有进程内事件总线
///
/// 作者：coisini
/// 创建时间：2026-07-16
pub struct AppState {
    /// Sidecar 生命周期控制器。
    pub lifecycle: Arc<SidecarLifecycle>,
    /// Agent sidecar 网关适配器。
    pub gateway: Arc<RuntimeAgentSidecar>,
    /// License 闸门（无锁 stub 或 verifier / fail-closed）。
    pub license: Arc<dyn LicenseGate>,
    /// In-process YouTube crawl jobs for the desktop UI.
    pub crawler: Arc<CrawlerService>,
    pub keywords_store: Arc<dyn CrawlerKeywordStore>,
    /// Accepted channels per job (`crawler_channel` SQLite table).
    pub channels_store: Arc<dyn CrawlerChannelStore>,
    /// Crawler key-value settings (`crawler_setting` SQLite table).
    pub settings_store: Arc<dyn CrawlerSettingsStore>,
    /// LLM provider metadata + keyring secrets.
    pub llm_settings_store: Arc<dyn LlmSettingsStore>,
    /// Business customer profiles (`customer` SQLite table in opendesk.db).
    pub customer_store: Arc<dyn CustomerStore>,
    /// Mail templates, accounts, and message history (`mail_*` tables in opendesk.db).
    pub mail_store: Arc<dyn MailStore>,
    /// Script snippet library (`script_snippet` table in opendesk.db).
    pub snippet_store: Arc<dyn ScriptSnippetStore>,
    #[allow(dead_code)]
    pub event_bus: Arc<InMemoryEventBus>,
}

/// 按 Cargo feature 构造 License 闸门。
///
/// 功能：
///
/// - 无 `license-lock`：返回 [`UnlockedLicenseGate`]
/// - 有 `license-lock`：优先 [`VerifierProcessLicense`]，失败则 fail-closed
///
/// 作者：coisini
/// 创建时间：2026-07-16
///
/// # 返回值
///
/// 返回可共享的 [`LicenseGate`] trait 对象。
pub fn build_license_gate() -> Arc<dyn LicenseGate> {
    #[cfg(feature = "license-lock")]
    {
        match VerifierProcessLicense::from_env() {
            Ok(gate) => Arc::new(gate),
            Err(error) => {
                tracing::error!(%error, "license-lock enabled but verifier unavailable");
                Arc::new(FailClosedLicenseGate::new(error.to_string()))
            }
        }
    }
    #[cfg(not(feature = "license-lock"))]
    {
        Arc::new(UnlockedLicenseGate::new())
    }
}
