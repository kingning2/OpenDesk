//! Tauri shell：组装 AppState、注册 IPC commands、启动 sidecar。
//!
//! Command 实现按域放在 [`commands`]；本文件只做进程启动与 wiring。
//!
//! 作者：coisini
//! 创建时间：2026-07-16

mod commands;
mod crawler_emit;
mod logging;
mod paths;
mod platform;
mod state;

use adapter::agent_sidecar::RuntimeAgentSidecar;
use commands::{
    agent_ping, crawler_job_cancel, crawler_job_logs, crawler_job_results, crawler_job_start,
    crawler_job_status, crawler_keywords_batches, crawler_keywords_import,
    crawler_youtube_api_key_get, crawler_youtube_api_key_set, customer_create, customer_get,
    customer_list, customer_update, license_activate, license_machine_code, license_status,
    llm_settings_get, llm_settings_save, llm_test_connection, mail_account_list, mail_account_save,
    mail_record_inbound, mail_send, mail_template_apply, mail_template_list,
    workflow_snippet_delete, workflow_snippet_list, workflow_snippet_save,
};
use crawler::{CrawlerService, CrawlerUiEmitter};
use crawler_emit::TauriCrawlerEmitter;
use kernel::event::{EventBus, InMemoryEventBus};
use logging::init_tracing;
use paths::{crawler_db_path, opendesk_db_path};
use ports::background_job::BackgroundJobStore;
use ports::crawler_channels::CrawlerChannelStore;
use ports::crawler_keywords::CrawlerKeywordStore;
use ports::crawler_settings::CrawlerSettingsStore;
use ports::customer::CustomerStore;
use runtime::sidecar::lifecycle::{SidecarConfig, SidecarLifecycle};
use state::{build_license_gate, AppState};
use std::sync::Arc;
use storage::background_job::SqliteBackgroundJobStore;
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

    let event_bus = Arc::new(InMemoryEventBus::new());
    let lifecycle = Arc::new(SidecarLifecycle::new(
        SidecarConfig::from_env(),
        event_bus.clone() as Arc<dyn EventBus>,
    ));
    let gateway = Arc::new(RuntimeAgentSidecar::new(lifecycle.client().clone()));
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
    crawler.attach_job_store(job_store);
    let keywords_store =
        Arc::new(SqliteCrawlerKeywordStore::new(crawler_db)) as Arc<dyn CrawlerKeywordStore>;
    let customer_store =
        Arc::new(SqliteCustomerStore::new(opendesk_db.clone())) as Arc<dyn CustomerStore>;
    let mail_store =
        Arc::new(SqliteMailStore::new(opendesk_db.clone())) as Arc<dyn ports::mail::MailStore>;
    let snippet_store = Arc::new(SqliteScriptSnippetStore::new(opendesk_db.clone()))
        as Arc<dyn ports::workflow::ScriptSnippetStore>;
    let app_state = AppState {
        lifecycle: lifecycle.clone(),
        gateway,
        license,
        crawler: crawler.clone(),
        keywords_store,
        channels_store,
        settings_store,
        llm_settings_store,
        customer_store,
        mail_store,
        snippet_store,
        event_bus,
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
            let lifecycle = state.lifecycle.clone();
            let llm_store = state.llm_settings_store.clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(env) = (|| {
                    let mut env = std::collections::HashMap::new();
                    let Some(record) = llm_store.get().map_err(|e| e.to_string())? else {
                        return Ok(env);
                    };
                    env.insert("OPENDESK_LLM_PROVIDER".into(), record.provider);
                    if let Some(base_url) = record.base_url {
                        env.insert("OPENDESK_LLM_BASE_URL".into(), base_url);
                    }
                    env.insert("OPENDESK_LLM_MODEL_ID".into(), record.model_id);
                    if let Some(api_key) = llm_store.resolve_api_key().map_err(|e| e.to_string())? {
                        env.insert("OPENDESK_LLM_API_KEY".into(), api_key);
                    }
                    Ok::<_, String>(env)
                })() {
                    lifecycle.set_process_env(env).await;
                }
                if let Err(error) = lifecycle.ensure_running().await {
                    tracing::error!(%error, "sidecar startup failed");
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            agent_ping,
            license_status,
            license_machine_code,
            license_activate,
            crawler_job_start,
            crawler_job_cancel,
            crawler_job_status,
            crawler_job_logs,
            crawler_job_results,
            crawler_keywords_import,
            crawler_keywords_batches,
            crawler_youtube_api_key_get,
            crawler_youtube_api_key_set,
            llm_settings_get,
            llm_settings_save,
            llm_test_connection,
            customer_list,
            customer_get,
            customer_create,
            customer_update,
            mail_template_list,
            mail_template_apply,
            mail_account_list,
            mail_account_save,
            mail_send,
            mail_record_inbound,
            workflow_snippet_list,
            workflow_snippet_save,
            workflow_snippet_delete
        ])
        .build(context)?
        .run(move |app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                let lifecycle = app_handle.state::<AppState>().lifecycle.clone();
                tauri::async_runtime::block_on(async move {
                    if let Err(error) = lifecycle.stop().await {
                        tracing::error!(%error, "sidecar shutdown failed");
                    }
                });
            }
        });

    Ok(())
}
