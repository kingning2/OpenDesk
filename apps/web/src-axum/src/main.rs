//! OpenDesk 独立 HTTP server：暴露桌面应用业务能力给 web 前端。
//!
//! 与桌面共享同一套 SQLite 数据文件；进程内 CrawlerService 与 SSE 事件中枢
//! 负责实时推送（爬虫进度 / chat token / 邮件同步 / 知识库导入）。

mod crawler;
mod knowledge;
mod llm;
mod rpc;
mod sse;
mod upload;

use app_core::build_app_state;
use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use ports::background_job::BackgroundJobStore;
use std::sync::Arc;

use crate::sse::{SseChatEmitter, SseCrawlerEmitter, SseHub};

/// Server 共享状态：应用状态 + SSE 中枢 + chat emitter。
#[derive(Clone)]
struct ServerState {
    app: Arc<app_core::AppState>,
    hub: SseHub,
    chat_emitter: SseChatEmitter,
}

/// 本机绑定地址；`OPENDESK_SERVER_BIND` / `OPENDESK_SERVER_PORT` 可覆盖。
fn bind_addr() -> std::net::SocketAddr {
    let host = std::env::var("OPENDESK_SERVER_BIND").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = std::env::var("OPENDESK_SERVER_PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(8899);
    format!("{host}:{port}")
        .parse()
        .unwrap_or_else(|_| "127.0.0.1:8899".parse().expect("static addr"))
}

/// SSE 事件端点：一个订阅者一个频道，帧格式 `event: <topic>\ndata: <json>`。
async fn sse_events(State(state): State<ServerState>) -> Response {
    let (rx, id) = state.hub.subscribe().await;
    let hub = state.hub;
    let stream = futures::stream::unfold((rx, hub, id, false), |(rx, hub, id, done)| async move {
        let mut rx = rx;
        if done {
            return None;
        }
        match rx.recv().await {
            Some(frame) => {
                let event = Event::default().data(frame.as_ref());
                Some((
                    Ok::<Event, std::convert::Infallible>(event),
                    (rx, hub, id, false),
                ))
            }
            None => {
                // 客户端断开或频道关闭：退订后结束流。
                hub.unsubscribe(id).await;
                Some((
                    Ok::<Event, std::convert::Infallible>(Event::default().data("bye")),
                    (rx, hub, id, true),
                ))
            }
        }
    });
    let sse = Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("keep-alive"),
    );
    sse.into_response()
}

async fn healthz() -> &'static str {
    "ok"
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(async_main())
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,opendesk_server=debug"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

async fn async_main() -> Result<(), Box<dyn std::error::Error>> {
    let (app_state, workflow_event_bus, crawler) = build_app_state();
    let app_state = Arc::new(app_state);

    // SSE 中枢 + 各 emitter 桥。
    let hub = SseHub::new();
    let chat_emitter = SseChatEmitter::new(hub.clone());
    let crawler_emitter = Arc::new(SseCrawlerEmitter::new(hub.clone()));
    crawler.attach_emitter(crawler_emitter);

    // Workflow 事件总线 → SSE。
    {
        let hub = hub.clone();
        workflow_event_bus.subscribe(Arc::new(move |event| {
            let payload = crate::workflow::event_to_phase(event);
            let frame = sse::encode_frame("workflow_runtime:phase", &payload);
            let hub = hub.clone();
            tokio::spawn(async move {
                hub.broadcast(frame).await;
            });
        }));
    }

    // 邮件同步 / 知识库导入状态推送循环。
    {
        let hub = hub.clone();
        let mail_store = app_state.mail_store.clone();
        let job_store = app_state.job_store.clone();
        tokio::spawn(watch_imap_sync_push(hub.clone(), mail_store, job_store));
        let knowledge_store = app_state.knowledge_store.clone();
        tokio::spawn(watch_knowledge_import_push(hub, knowledge_store));
    }

    // IMAP 周期调度。
    {
        let job_store = app_state.job_store.clone();
        let mail_store = app_state.mail_store.clone();
        let customer_store = app_state.customer_store.clone();
        tokio::spawn(async move {
            let interval_secs = std::env::var("OPENDESK_IMAP_SYNC_INTERVAL_SECS")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(180);
            let mut ticker = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
            loop {
                ticker.tick().await;
                let job_store = job_store.clone();
                let mail_store = mail_store.clone();
                let customer_store = customer_store.clone();
                let result = tokio::task::spawn_blocking(move || {
                    mail::app::ScheduleImapSync::execute(
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
    }

    // 尽力拉起 opendesk-worker（单实例文件锁自行仲裁）。
    spawn_worker(app_state.clone());

    let server_state = ServerState {
        app: app_state.clone(),
        hub: hub.clone(),
        chat_emitter,
    };

    let router = Router::new()
        .route("/healthz", get(healthz))
        .route("/api/rpc", post(rpc::rpc))
        .route("/api/events", get(sse_events))
        .route(
            "/api/upload",
            post(upload::upload_file).layer(upload::default_body_limit()),
        )
        .with_state(server_state);

    let addr = bind_addr();
    tracing::info!(%addr, "opendesk-server listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal(app_state.clone()))
        .await?;

    Ok(())
}

/// ctrl_c 时停止 worker 子进程并退出。
async fn shutdown_signal(app_state: Arc<app_core::AppState>) {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutting down");
    if let Some(mut child) = app_state.worker.lock().expect("worker mutex").take() {
        let _ = child.kill();
        let _ = child.wait();
        tracing::info!("opendesk-worker stopped");
    }
}

/// 邮件同步状态推送循环（与桌面 watch_imap_sync_push 同构，改用 SSE）。
async fn watch_imap_sync_push(
    hub: SseHub,
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
        tokio::task::spawn_blocking(move || {
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
        for account_id in snapshot.keys() {
            if last.get(account_id) != snapshot.get(account_id) {
                let frame = sse::encode_frame("mail:imap-sync-updated", account_id);
                hub.broadcast(frame).await;
            }
        }
        last = snapshot;
    }
}

/// 知识库导入状态推送循环（与桌面 watch_knowledge_import_push 同构，改用 SSE）。
async fn watch_knowledge_import_push(
    hub: SseHub,
    knowledge_store: Arc<dyn ports::knowledge::KnowledgeStore>,
) {
    use std::collections::HashMap;
    use std::time::Duration;

    type Fingerprint = (String, String);

    async fn capture(
        knowledge_store: Arc<dyn ports::knowledge::KnowledgeStore>,
    ) -> HashMap<String, Fingerprint> {
        tokio::task::spawn_blocking(move || {
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
        for (document_id, (status, _)) in &snapshot {
            let changed = last
                .get(document_id)
                .map(|(old_status, _)| old_status != status)
                .unwrap_or(true);
            if changed {
                let payload = common::contracts::KnowledgeEventImportUpdated {
                    document_id: document_id.clone(),
                    status: status.clone(),
                    error_message: None,
                };
                let frame = sse::encode_frame("knowledge:import/updated", &payload);
                hub.broadcast(frame).await;
            }
        }
        last = snapshot;
    }
}

/// 拉起 opendesk-worker（尽力而为；找不到二进制仅告警）。
fn spawn_worker(app_state: Arc<app_core::AppState>) {
    let Some(path) = find_worker_binary() else {
        tracing::warn!("opendesk-worker binary not found; mail sync/idle disabled");
        return;
    };
    match std::process::Command::new(&path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(child) => {
            *app_state.worker.lock().expect("worker mutex") = Some(child);
            tracing::info!(?path, "opendesk-worker spawned");
        }
        Err(error) => {
            tracing::error!(%error, ?path, "failed to spawn opendesk-worker");
        }
    }
}

/// 查找 worker 二进制（与桌面 `find_worker_binary` 同构）。
fn find_worker_binary() -> Option<std::path::PathBuf> {
    let triple = std::env::var("OPENDESK_WORKER_TARGET_TRIPLE").unwrap_or_default();
    let exe = if triple.contains("windows") || cfg!(windows) {
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

/// 把 Workflow 事件转为 SSE 载荷（与桌面 workflow_runtime_emit 同构）。
mod workflow {
    use common::contracts::WorkflowRuntimeEventPhase;
    use workflow_runtime::{NodeState, WorkflowEvent, WorkflowState};

    pub fn event_to_phase(event: &WorkflowEvent) -> WorkflowRuntimeEventPhase {
        match event {
            WorkflowEvent::WorkflowStarted { instance_id, state } => WorkflowRuntimeEventPhase {
                kind: event.name().to_string(),
                instance_id: instance_id.as_str().to_string(),
                node_id: None,
                state: Some(state.as_str().to_string()),
                message: None,
                context_version: None,
            },
            WorkflowEvent::WorkflowCompleted { instance_id } => WorkflowRuntimeEventPhase {
                kind: event.name().to_string(),
                instance_id: instance_id.as_str().to_string(),
                node_id: None,
                state: Some(WorkflowState::Completed.as_str().to_string()),
                message: None,
                context_version: None,
            },
            WorkflowEvent::WorkflowFailed {
                instance_id,
                message,
            } => WorkflowRuntimeEventPhase {
                kind: event.name().to_string(),
                instance_id: instance_id.as_str().to_string(),
                node_id: None,
                state: Some(WorkflowState::Failed.as_str().to_string()),
                message: Some(message.clone()),
                context_version: None,
            },
            WorkflowEvent::WorkflowPaused { instance_id } => WorkflowRuntimeEventPhase {
                kind: event.name().to_string(),
                instance_id: instance_id.as_str().to_string(),
                node_id: None,
                state: Some(WorkflowState::Paused.as_str().to_string()),
                message: None,
                context_version: None,
            },
            WorkflowEvent::WorkflowCancelled { instance_id } => WorkflowRuntimeEventPhase {
                kind: event.name().to_string(),
                instance_id: instance_id.as_str().to_string(),
                node_id: None,
                state: Some(WorkflowState::Cancelled.as_str().to_string()),
                message: None,
                context_version: None,
            },
            WorkflowEvent::NodeStarted {
                instance_id,
                node_id,
            } => WorkflowRuntimeEventPhase {
                kind: event.name().to_string(),
                instance_id: instance_id.as_str().to_string(),
                node_id: Some(node_id.as_str().to_string()),
                state: Some(NodeState::Running.as_str().to_string()),
                message: None,
                context_version: None,
            },
            WorkflowEvent::NodeCompleted {
                instance_id,
                node_id,
                state,
            } => WorkflowRuntimeEventPhase {
                kind: event.name().to_string(),
                instance_id: instance_id.as_str().to_string(),
                node_id: Some(node_id.as_str().to_string()),
                state: Some(state.as_str().to_string()),
                message: None,
                context_version: None,
            },
            WorkflowEvent::NodeFailed {
                instance_id,
                node_id,
                message,
            } => WorkflowRuntimeEventPhase {
                kind: event.name().to_string(),
                instance_id: instance_id.as_str().to_string(),
                node_id: Some(node_id.as_str().to_string()),
                state: Some(NodeState::Failed.as_str().to_string()),
                message: Some(message.clone()),
                context_version: None,
            },
            WorkflowEvent::NodeRetryScheduled {
                instance_id,
                node_id,
                ..
            } => WorkflowRuntimeEventPhase {
                kind: event.name().to_string(),
                instance_id: instance_id.as_str().to_string(),
                node_id: Some(node_id.as_str().to_string()),
                state: Some(NodeState::RetryWaiting.as_str().to_string()),
                message: None,
                context_version: None,
            },
            WorkflowEvent::ContextChanged {
                instance_id,
                version,
            } => WorkflowRuntimeEventPhase {
                kind: event.name().to_string(),
                instance_id: instance_id.as_str().to_string(),
                node_id: None,
                state: None,
                message: None,
                context_version: Some(*version as i64),
            },
        }
    }
}
