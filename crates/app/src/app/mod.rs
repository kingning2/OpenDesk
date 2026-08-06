//! Tauri shell：组装 AppState、注册 IPC commands。
//!
//! Command 实现按域放在 [`commands`]；本模块只做进程启动与 wiring。
//!
//! 作者：coisini
//! 创建时间：2026-07-16

mod chat_emit;
mod chat_tools;
mod commands;
mod crawler_emit;
mod logging;
mod paths;
mod platform;
mod state;

use crawler::{CrawlerService, CrawlerUiEmitter};
use crawler_emit::TauriCrawlerEmitter;
use logging::init_tracing;
use mail::app::ScheduleImapSync;
use paths::{chat_db_path, crawler_db_path, embedding_cache_dir, opendesk_db_path};
use ports::background_job::BackgroundJobStore;
use ports::chat::{ChatMemoryStore, ChatStore};
use ports::crawler_channels::CrawlerChannelStore;
use ports::crawler_keywords::CrawlerKeywordStore;
use ports::crawler_settings::CrawlerSettingsStore;
use ports::customer::CustomerStore;
use state::{build_license_gate, AppState};
use std::sync::Arc;
use storage::background_job::SqliteBackgroundJobStore;
use storage::chat::SqliteChatStore;
use storage::crawler_channels::SqliteCrawlerChannelStore;
use storage::crawler_db::CrawlerDb;
use storage::crawler_keywords::SqliteCrawlerKeywordStore;
use storage::crawler_settings::SqliteCrawlerSettingsStore;
use storage::customer::SqliteCustomerStore;
use storage::llm_settings::SqliteLlmSettingsStore;
use storage::mail::SqliteMailStore;
use storage::opendesk_db::OpendeskDb;
use storage::workflow::SqliteScriptSnippetStore;
use tauri::Manager;

/// 启动桌面应用：打开数据库、挂载 crawler emitter、注册 IPC、运行事件循环。
///
/// 作者：coisini
/// 创建时间：2026-07-16
///
/// # 参数
/// - `context` — Tauri 构建上下文
///
/// # 返回值
/// 事件循环结束后的 `tauri::Result`。
pub fn launch(context: tauri::Context<tauri::Wry>) -> tauri::Result<()> {
    init_tracing();

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
    let llm_settings_store = Arc::new(SqliteLlmSettingsStore::new(opendesk_db.clone()))
        as Arc<dyn ports::llm_settings::LlmSettingsStore>;
    let crawler = Arc::new(CrawlerService::new(channels_store.clone()));
    crawler.attach_job_store(job_store.clone());
    let keywords_store =
        Arc::new(SqliteCrawlerKeywordStore::new(crawler_db)) as Arc<dyn CrawlerKeywordStore>;
    let customer_store =
        Arc::new(SqliteCustomerStore::new(opendesk_db.clone())) as Arc<dyn CustomerStore>;
    let mail_store =
        Arc::new(SqliteMailStore::new(opendesk_db.clone())) as Arc<dyn ports::mail::MailStore>;
    let snippet_store = Arc::new(SqliteScriptSnippetStore::new(opendesk_db.clone()))
        as Arc<dyn ports::workflow::ScriptSnippetStore>;
    let chat = Arc::new(SqliteChatStore::open(chat_db_path()).expect("open chat database"));
    let chat_store = Arc::clone(&chat) as Arc<dyn ChatStore>;
    let chat_memory_store = Arc::clone(&chat) as Arc<dyn ChatMemoryStore>;
    let embedder = Arc::new(agent::embedding::EmbeddingService::new(Some(
        embedding_cache_dir(),
    ))) as Arc<dyn agent::embedding::Embedder>;
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
        snippet_store,
        chat_store,
        chat_memory_store,
        embedder,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .append_invoke_initialization_script(platform::platform_initialization_script())
        .manage(app_state)
        .setup(move |app| {
            let emitter = Arc::new(TauriCrawlerEmitter::new(app.handle().clone()))
                as Arc<dyn CrawlerUiEmitter>;
            crawler.attach_emitter(emitter);

            let state = app.state::<AppState>();

            // 预热嵌入模型：首次启动在后台联网下载 bge-small-zh-v1.5（~100MB）与
            // onnxruntime 到缓存目录，之后完全离线。失败仅告警，首次记忆检索会惰性重试。
            let warmup_embedder = state.embedder.clone();
            tauri::async_runtime::spawn(async move {
                let result =
                    tauri::async_runtime::spawn_blocking(move || warmup_embedder.preload()).await;
                match result {
                    Ok(Ok(())) => tracing::info!("embedding model ready"),
                    Ok(Err(error)) => {
                        tracing::warn!(%error, "embedding model warmup failed; will retry on first memory use")
                    }
                    Err(error) => {
                        tracing::warn!(%error, "embedding model warmup task join failed")
                    }
                }
            });

            let imap_job_store = state.job_store.clone();
            let imap_mail_store = state.mail_store.clone();
            let imap_customer_store = state.customer_store.clone();
            tauri::async_runtime::spawn(async move {
                let interval_secs = std::env::var("OPENDESK_IMAP_SYNC_INTERVAL_SECS")
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(180);
                let mut ticker =
                    tokio::time::interval(std::time::Duration::from_secs(interval_secs));
                loop {
                    ticker.tick().await;
                    let job_store = imap_job_store.clone();
                    let mail_store = imap_mail_store.clone();
                    let customer_store = imap_customer_store.clone();
                    let result = tauri::async_runtime::spawn_blocking(move || {
                        ScheduleImapSync::execute(
                            job_store.as_ref(),
                            mail_store.as_ref(),
                            customer_store.as_ref(),
                        )
                    })
                    .await;
                    if let Err(error) = result {
                        tracing::warn!(%error, "imap periodic scheduler join failed");
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::agent::agent_ping,
            commands::chat::chat_send,
            commands::chat::chat_session_list,
            commands::chat::chat_session_create,
            commands::chat::chat_session_rename,
            commands::chat::chat_session_delete,
            commands::chat::chat_messages_load,
            commands::license::license_status,
            commands::license::license_machine_code,
            commands::license::license_activate,
            commands::crawler::job::crawler_job_start,
            commands::crawler::job::crawler_job_cancel,
            commands::crawler::job::crawler_job_status,
            commands::crawler::job::crawler_job_logs,
            commands::crawler::job::crawler_job_results,
            commands::crawler::channels::crawler_channel_list,
            commands::crawler::channels::crawler_channel_update,
            commands::crawler::keywords::crawler_keywords_import,
            commands::crawler::keywords::crawler_keywords_batches,
            commands::crawler::keywords::crawler_keywords_generate,
            commands::crawler::settings::crawler_youtube_api_key_get,
            commands::crawler::settings::crawler_youtube_api_key_set,
            commands::llm::llm_settings_get,
            commands::llm::llm_settings_save,
            commands::llm::llm_test_connection,
            commands::customer::customer_list,
            commands::customer::customer_get,
            commands::customer::customer_create,
            commands::customer::customer_update,
            commands::mail::mail_template_list,
            commands::mail::mail_template_save,
            commands::mail::mail_template_apply,
            commands::mail::mail_account_list,
            commands::mail::mail_account_save,
            commands::mail::mail_message_list,
            commands::mail::mail_generate_html,
            commands::mail::mail_send,
            commands::mail::mail_record_inbound,
            commands::mail::mail_sync_now,
            commands::mail::mail_sync_status,
            commands::mail::mail_inbox_unmatched_list,
            commands::mail::mail_link_inbound_customer,
            commands::mail_integration::mail_email_read_integration_get,
            commands::mail_integration::mail_email_read_integration_save,
            commands::mail_integration::mail_email_read_integration_probe,
            commands::workflow::workflow_snippet_list,
            commands::workflow::workflow_snippet_save,
            commands::workflow::workflow_snippet_delete
        ])
        .build(context)?
        .run(|_, _| {});

    Ok(())
}
