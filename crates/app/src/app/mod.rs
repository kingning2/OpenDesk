//! Tauri shell：组装 AppState、注册 IPC commands。
//!
//! Command 实现按域放在 [`commands`]；本模块只做进程启动与 wiring。
//!
//! 作者：coisini
//! 创建时间：2026-07-16

mod chat_emit;
mod chat_skills;
mod chat_tools;
mod commands;
mod crawler_emit;
mod logging;
mod platform;
mod state;
mod workflow_runtime_emit;

use crawler::youtube::CrawlerUIEmitter;
use crawler_emit::TauriCrawlerEmitter;
use logging::init_tracing;
use mail::app::ScheduleImapSync;
use ports::background_job::BackgroundJobStore;
use state::AppState;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use workflow_runtime_emit::TauriWorkflowRuntimeEmitter;

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

    let (app_state, workflow_event_bus, crawler) = app_core::build_app_state();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .append_invoke_initialization_script(platform::platform_initialization_script())
        .manage(app_state)
        .setup(move |app| {
            let emitter = Arc::new(TauriCrawlerEmitter::new(app.handle().clone()))
                as Arc<dyn CrawlerUIEmitter>;
            crawler.attach_emitter(emitter);

            let workflow_emitter = Arc::new(TauriWorkflowRuntimeEmitter::new(app.handle().clone()));
            let workflow_emitter_for_sub = Arc::clone(&workflow_emitter);
            workflow_event_bus.subscribe(Arc::new(move |event| {
                workflow_emitter_for_sub.emit_phase(event);
            }));

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

            // 邮件同步状态推送：同步在独立 worker 进程写库，主进程低频检测 `mail_imap_sync_state`
            // 与 `background_job` 的变化并 emit 到 webview，让前端免轮询刷新。
            let push_app = app.handle().clone();
            let push_mail_store = state.mail_store.clone();
            let push_job_store = state.job_store.clone();
            tauri::async_runtime::spawn(watch_imap_sync_push(
                push_app,
                push_mail_store,
                push_job_store,
            ));

            // 知识库导入状态推送：导入在独立 worker 进程写 knowledge.db，主进程低频检测
            // `knowledge_doc` 的状态变化并 emit 到 webview，让前端免轮询刷新。
            let kb_push_app = app.handle().clone();
            let kb_knowledge_store = state.knowledge_store.clone();
            tauri::async_runtime::spawn(watch_knowledge_import_push(kb_push_app, kb_knowledge_store));

            // 自动拉起 opendesk-worker：IMAP 收信与后台任务都在独立 worker 进程执行，
            // 这里保证它随主进程一起启动。worker 自身带单实例文件锁，避免重复拉起。
            let worker_handle = state.worker.clone();
            match find_worker_binary() {
                Some(path) => {
                    match std::process::Command::new(&path)
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .spawn()
                    {
                        Ok(child) => {
                            *worker_handle.lock().expect("worker mutex") = Some(child);
                            tracing::info!(
                                target: "lifecycle",
                                ?path,
                                "opendesk-worker spawned"
                            );
                        }
                        Err(error) => {
                            tracing::error!(
                                target: "lifecycle",
                                %error,
                                ?path,
                                "failed to spawn opendesk-worker"
                            );
                        }
                    }
                }
                None => {
                    tracing::warn!(
                        target: "lifecycle",
                        "opendesk-worker binary not found; mail sync/idle disabled (run pnpm build:worker)"
                    );
                }
            }
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
            commands::help::help_ask,
            commands::knowledge::knowledge_doc_import,
            commands::knowledge::knowledge_doc_list,
            commands::knowledge::knowledge_doc_delete,
            commands::knowledge::knowledge_tool_status,
            commands::knowledge::knowledge_tool_download,
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
            commands::dashboard::dashboard_stats,
            commands::mail::mail_template_list,
            commands::mail::mail_template_save,
            commands::mail::mail_template_apply,
            commands::mail::mail_account_list,
            commands::mail::mail_account_save,
            commands::mail::mail_account_delete,
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
            commands::workflow::workflow_template_list,
            commands::workflow::workflow_template_get,
            commands::workflow::workflow_binding_list,
            commands::workflow::workflow_rule_list,
            commands::workflow::workflow_script_list,
            commands::workflow_runtime::workflow_runtime_start,
            commands::workflow_runtime::workflow_runtime_cancel,
            commands::workflow_runtime::workflow_runtime_resume,
            commands::workflow_runtime::workflow_runtime_active
        ])
        .build(context)?
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                if let Some(state) = app.try_state::<AppState>() {
                    if let Some(mut child) = state.worker.lock().expect("worker mutex").take() {
                        let _ = child.kill();
                        let _ = child.wait();
                        tracing::info!(target: "lifecycle", "opendesk-worker stopped");
                    }
                }
            }
        });

    Ok(())
}

/// 邮件同步状态推送循环：每秒检测各账号 sync 状态（last_sync_at / last_error / is_syncing），
/// 有变化即 emit `mail:imap-sync-updated`。worker 是独立进程、无法直接访问 Tauri，只能靠
/// 主进程读共享数据库这座桥把更新推给 webview。
async fn watch_imap_sync_push(
    app: tauri::AppHandle,
    mail_store: Arc<dyn ports::mail::MailStore>,
    job_store: Arc<dyn BackgroundJobStore>,
) {
    use std::collections::HashMap;
    use std::time::Duration;

    type Fingerprint = (Option<String>, Option<String>, bool);

    async fn capture(
        mail_store: Arc<dyn ports::mail::MailStore>,
        job_store: Arc<dyn BackgroundJobStore>,
    ) -> HashMap<String, Fingerprint> {
        tauri::async_runtime::spawn_blocking(move || {
            let mut map: HashMap<String, Fingerprint> = HashMap::new();
            let Ok(states) = mail_store.list_imap_sync_states(None) else {
                return map;
            };
            for state in states {
                let syncing = job_store
                    .has_active_imap_sync(&state.account_id)
                    .unwrap_or(false);
                map.insert(
                    state.account_id,
                    (state.last_sync_at, state.last_error, syncing),
                );
            }
            map
        })
        .await
        .unwrap_or_default()
    }

    let mut last = capture(mail_store.clone(), job_store.clone()).await;
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let snapshot = capture(mail_store.clone(), job_store.clone()).await;
        for (account_id, fingerprint) in &snapshot {
            if last.get(account_id) != Some(fingerprint) {
                let _ = app.emit("mail:imap-sync-updated", account_id);
            }
        }
        last = snapshot;
    }
}

/// 知识库导入状态推送循环：每秒检测 `knowledge_doc` 的状态（parsing → ready/failed），
/// 有变化即 emit `knowledge:import/updated`。worker 是独立进程无法直接访问 Tauri，
/// 靠主进程读共享 knowledge.db 把导入完成/失败推给 webview。
async fn watch_knowledge_import_push(
    app: tauri::AppHandle,
    knowledge_store: Arc<dyn ports::knowledge::KnowledgeStore>,
) {
    use std::collections::HashMap;
    use std::time::Duration;

    type Fingerprint = (String, String);

    async fn capture(
        knowledge_store: Arc<dyn ports::knowledge::KnowledgeStore>,
    ) -> HashMap<String, Fingerprint> {
        tauri::async_runtime::spawn_blocking(move || {
            let mut map: HashMap<String, Fingerprint> = HashMap::new();
            let Ok(documents) = knowledge_store.list_documents() else {
                return map;
            };
            for doc in documents {
                map.insert(doc.id, (doc.status, doc.updated_at.to_string()));
            }
            map
        })
        .await
        .unwrap_or_default()
    }

    let mut last = capture(knowledge_store.clone()).await;
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let snapshot = capture(knowledge_store.clone()).await;
        // 逐条比较：新出现的文档（last 没有）或状态变化的文档推事件。
        for (document_id, (status, _)) in &snapshot {
            let changed = last
                .get(document_id)
                .map(|(old_status, _)| old_status != status)
                .unwrap_or(true);
            if changed {
                let _ = app.emit(
                    "knowledge:import/updated",
                    common::contracts::KnowledgeEventImportUpdated {
                        document_id: document_id.clone(),
                        status: status.clone(),
                        error_message: None,
                    },
                );
            }
        }
        last = snapshot;
    }
}

/// 查找 `opendesk-worker` 可执行文件：优先打包后的 sidecar（主程序旁、带 triple 后缀），
/// 其次构建脚本产物 `apps/desktop/src-tauri/binaries` 与 Cargo target 目录。
/// 跳过 build.rs 生成的 1 字节 dev stub。
fn find_worker_binary() -> Option<std::path::PathBuf> {
    let triple = env!("OPENDESK_WORKER_TARGET_TRIPLE");
    let exe = if triple.contains("windows") {
        ".exe"
    } else {
        ""
    };
    let name = format!("opendesk-worker{exe}");
    let bundled = format!("opendesk-worker-{triple}{exe}");

    let mut dirs: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(current) = std::env::current_exe() {
        if let Some(dir) = current.parent() {
            dirs.push(dir.to_path_buf());
        }
    }
    if let Ok(mut dir) = std::env::current_dir() {
        for _ in 0..6 {
            dirs.push(dir.clone());
            let binaries = dir.join("apps/desktop/src-tauri/binaries");
            if binaries.is_dir() {
                dirs.push(binaries);
            }
            if !dir.pop() {
                break;
            }
        }
    }

    for dir in dirs {
        for candidate in [dir.join(&bundled), dir.join(&name)] {
            if is_real_worker(&candidate) {
                return Some(candidate);
            }
        }
        for sub in [
            format!("target/{triple}/debug"),
            format!("target/{triple}/release"),
            "target/debug".to_string(),
            "target/release".to_string(),
        ] {
            let candidate = dir.join(sub).join(&name);
            if is_real_worker(&candidate) {
                return Some(candidate);
            }
        }
    }
    None
}

fn is_real_worker(path: &std::path::Path) -> bool {
    std::fs::metadata(path)
        .map(|meta| meta.is_file() && meta.len() > 2)
        .unwrap_or(false)
}
