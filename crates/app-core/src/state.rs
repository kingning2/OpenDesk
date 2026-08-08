//! 无 Tauri 的应用运行时状态：store / 服务装配与共享状态结构。
//!
//! 作者：coisini
//! 创建时间：2026-08-08

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
use ports::workflow_runtime::CheckpointStore;
use std::process::Child;
use std::sync::{Arc, Mutex};
use workflow_runtime::{
    register_builtin_executors, ExecutorRegistry, InMemoryEventBus, SchedulerConfig,
    WorkflowRuntimeFacade,
};

use crate::paths::{
    chat_db_path, crawler_db_path, embedding_cache_dir, knowledge_db_path, opendesk_db_path,
};
use storage::background_job::SqliteBackgroundJobStore;
use storage::chat::SqliteChatStore;
use storage::crawler_channels::SqliteCrawlerChannelStore;
use storage::crawler_db::CrawlerDb;
use storage::crawler_keywords::SqliteCrawlerKeywordStore;
use storage::crawler_settings::SqliteCrawlerSettingsStore;
use storage::customer::SqliteCustomerStore;
use storage::knowledge::SqliteKnowledgeStore;
use storage::llm_settings::SqliteLlmSettingsStore;
use storage::mail::SqliteMailStore;
use storage::opendesk_db::OpendeskDb;
use storage::workflow::SqliteWorkflowStore;
use storage::workflow_runtime::SqliteCheckpointStore;

/// 应用共享状态（Tauri 与 HTTP server 进程共用）。
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
    /// Workflow account→template bindings / rules / scripts (`workflow_*` tables).
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
/// - 无 `license-lock`：返回 [`UnlockedLicenseGate`]
/// - 有 `license-lock`：优先 [`VerifierProcessLicense`]，失败则 fail-closed
///
/// 作者：coisini
/// 创建时间：2026-07-16
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

/// 装配完整 `AppState`：打开 SQLite、构建全部 store 与服务。
///
/// 返回 `(AppState, workflow 事件总线, CrawlerService)`——crawler 与事件总线
/// 需要由调用方在构造后 attach emitter（Tauri 用 Tauri 桥，HTTP server 用 SSE）。
///
/// 作者：coisini
/// 创建时间：2026-08-08
pub fn build_app_state() -> (AppState, Arc<InMemoryEventBus>, Arc<CrawlerService>) {
    let license = build_license_gate();
    let db_path = crawler_db_path();
    let opendesk_db = OpendeskDb::open(opendesk_db_path()).expect("open opendesk database");
    let job_store =
        Arc::new(SqliteBackgroundJobStore::new(opendesk_db.clone())) as Arc<dyn BackgroundJobStore>;
    let crawler_db = CrawlerDb::open(&db_path).expect("open crawler database");
    let channels_store = Arc::new(SqliteCrawlerChannelStore::new(crawler_db.clone()))
        as Arc<dyn CrawlerChannelStore>;
    let settings_store = Arc::new(SqliteCrawlerSettingsStore::new(crawler_db.clone()))
        as Arc<dyn CrawlerSettingsStore>;
    let llm_settings_store =
        Arc::new(SqliteLlmSettingsStore::new(opendesk_db.clone())) as Arc<dyn LlmSettingsStore>;
    let crawler = Arc::new(CrawlerService::new(channels_store.clone()));
    crawler.attach_job_store(job_store.clone());
    let keywords_store =
        Arc::new(SqliteCrawlerKeywordStore::new(crawler_db)) as Arc<dyn CrawlerKeywordStore>;
    let customer_store =
        Arc::new(SqliteCustomerStore::new(opendesk_db.clone())) as Arc<dyn CustomerStore>;
    let mail_store = Arc::new(SqliteMailStore::new(opendesk_db.clone())) as Arc<dyn MailStore>;
    let workflow_store =
        Arc::new(SqliteWorkflowStore::new(opendesk_db.clone())) as Arc<dyn WorkflowStore>;
    let chat = Arc::new(SqliteChatStore::open(chat_db_path()).expect("open chat database"));
    let chat_store = Arc::clone(&chat) as Arc<dyn ChatStore>;
    let chat_memory_store = Arc::clone(&chat) as Arc<dyn ChatMemoryStore>;
    let knowledge_store =
        Arc::new(SqliteKnowledgeStore::open(knowledge_db_path()).expect("open knowledge database"))
            as Arc<dyn KnowledgeStore>;
    let embedder = Arc::new(agent::embedding::EmbeddingService::new(Some(
        embedding_cache_dir(),
    ))) as Arc<dyn Embedder>;
    let skill_registry = Arc::new(agent::skills::system::system_registry());
    let mut workflow_registry = ExecutorRegistry::new();
    register_builtin_executors(&mut workflow_registry).expect("register workflow executors");
    let workflow_checkpoint =
        Arc::new(SqliteCheckpointStore::new(opendesk_db.clone())) as Arc<dyn CheckpointStore>;
    let workflow_event_bus = Arc::new(InMemoryEventBus::new());
    let workflow_runtime = Arc::new(WorkflowRuntimeFacade::new(
        workflow_registry,
        workflow_checkpoint,
        workflow_event_bus.clone(),
        SchedulerConfig::default(),
    ));
    let app_state = AppState {
        license,
        crawler: crawler.clone(),
        keywords_store,
        channels_store,
        settings_store,
        llm_settings_store,
        customer_store,
        mail_store,
        job_store: job_store.clone(),
        workflow_store,
        chat_store,
        chat_memory_store,
        knowledge_store,
        embedder,
        skill_registry,
        workflow_runtime,
        worker: Arc::new(Mutex::new(None)),
    };

    (app_state, workflow_event_bus, crawler)
}
