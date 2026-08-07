//! 应用运行时状态组装。
//!
//! 作者：coisini
//! 创建时间：2026-07-16

#[cfg(not(feature = "license-lock"))]
use adapter::license::UnlockedLicenseGate;
#[cfg(feature = "license-lock")]
use adapter::license::{FailClosedLicenseGate, VerifierProcessLicense};
use agent::embedding::Embedder;
use agent::skills::SkillRegistry;
use crawler::youtube::CrawlerService;
use ports::background_job::BackgroundJobStore;
use ports::chat::{ChatMemoryStore, ChatStore};
use ports::crawler_channels::CrawlerChannelStore;
use ports::crawler_keywords::CrawlerKeywordStore;
use ports::crawler_settings::CrawlerSettingsStore;
use ports::customer::CustomerStore;
use ports::knowledge::KnowledgeStore;
use ports::license::LicenseGate;
use ports::llm_settings::LlmSettingsStore;
use ports::mail::MailStore;
use ports::workflow::WorkflowStore;
use std::process::Child;
use std::sync::{Arc, Mutex};
use workflow_runtime::WorkflowRuntimeFacade;

/// 桌面应用共享状态。
///
/// 功能：
///
/// - 持有 License 闸门实现
/// - 持有进程内 crawler 与 SQLite stores
///
/// 作者：coisini
/// 创建时间：2026-07-16
pub struct AppState {
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
    /// Background job queue shared with `opendesk-worker`.
    pub job_store: Arc<dyn BackgroundJobStore>,
    /// Workflow definitions (templates, bindings, stages, rules, scripts) in opendesk.db.
    pub workflow_store: Arc<dyn WorkflowStore>,
    /// Chat sessions + messages (`chat.db`, rusqlite-backed).
    pub chat_store: Arc<dyn ChatStore>,
    /// Long-term memory + vector search (`chat.db` sqlite-vec, shares connection with chat_store).
    pub chat_memory_store: Arc<dyn ChatMemoryStore>,
    /// Knowledge base documents + vector search (`knowledge.db`, sqlite-vec).
    pub knowledge_store: Arc<dyn KnowledgeStore>,
    /// Local embedding service (fastembed, lazy model load).
    pub embedder: Arc<dyn Embedder>,
    /// 内置系统操作指引 Skill 注册表（注入聊天上下文，让 AI 了解系统）。
    pub skill_registry: Arc<SkillRegistry>,
    /// 通用工作流运行时（DAG 调度 + 检查点）。
    pub workflow_runtime: Arc<WorkflowRuntimeFacade>,
    /// 主进程自动拉起的 `opendesk-worker` 子进程（退出时清理）。
    pub worker: Arc<Mutex<Option<Child>>>,
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
