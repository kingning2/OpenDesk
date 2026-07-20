use adapter::agent_sidecar::RuntimeAgentSidecar;
use crawler::CrawlerService;
use kernel::event::InMemoryEventBus;
use ports::crawler_channels::CrawlerChannelStore;
use ports::crawler_keywords::CrawlerKeywordStore;
use ports::crawler_settings::CrawlerSettingsStore;
use runtime::sidecar::lifecycle::SidecarLifecycle;
use std::sync::Arc;

pub struct AppState {
    pub lifecycle: Arc<SidecarLifecycle>,
    /// Agent-only sidecar gateway (crawler no longer uses Python).
    pub gateway: Arc<RuntimeAgentSidecar>,
    /// In-process YouTube crawl jobs for the desktop UI.
    pub crawler: Arc<CrawlerService>,
    pub keywords_store: Arc<dyn CrawlerKeywordStore>,
    /// Accepted channels per job (`crawler_channel` SQLite table).
    pub channels_store: Arc<dyn CrawlerChannelStore>,
    /// Crawler key-value settings (`crawler_setting` SQLite table).
    pub settings_store: Arc<dyn CrawlerSettingsStore>,
    #[allow(dead_code)]
    pub event_bus: Arc<InMemoryEventBus>,
}
