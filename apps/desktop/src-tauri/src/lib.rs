//! Tauri shell：组装 AppState、注册 IPC commands、启动 sidecar。
//!
//! Command 实现按域放在 [`commands`]；本文件只做进程启动与 wiring。
//!
//! 作者：coisini
//! 创建时间：2026-07-16

mod agent;
mod ai_config;
mod channels;
mod commands;
mod logging;
mod platform;
mod state;
mod timing;

use adapter::agent_sidecar::RuntimeAgentSidecar;
use channels::commands::{
    channel_close_site, channel_connect, channel_disconnect, channel_open_site, channel_qr_cancel,
    channel_qr_check, channel_qr_start, channel_send, channel_state_get, channel_state_set,
};
use channels::coordinator::ChannelCoordinator;
use channels::dispatcher::ChannelDispatcher;
use channels::protocol::ChannelProtocol;
use channels::store::ChannelRepo;
use channels::xianyu::XianyuChannel;
use commands::{
    agent_ping, ai_config_get, ai_config_set, ai_test_api_key, license_activate,
    license_machine_code, license_status, log_clear, log_recent,
};
use kernel::event::{EventBus, InMemoryEventBus};
use logging::init_tracing;
use runtime::sidecar::lifecycle::{SidecarConfig, SidecarLifecycle};
use state::{build_license_gate, AppState};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;

/// 启动桌面应用：组装 AppState、注册 IPC、运行事件循环。
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
    let app_state = AppState {
        lifecycle: lifecycle.clone(),
        gateway,
        license,
        event_bus,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .append_invoke_initialization_script(platform::platform_initialization_script())
        .manage(app_state)
        .setup(move |app| {
            let config_dir = match app.path().app_config_dir() {
                Ok(dir) => dir,
                Err(error) => {
                    tracing::error!(%error, "解析应用配置目录失败；AI 配置已禁用");
                    PathBuf::from(".")
                }
            };
            app.manage(Arc::new(ai_config::AiConfigStore::new(config_dir.clone())));

            // 渠道层：SQLite + 调度器 + 协调器。
            let db_dir = config_dir.join("channel");
            std::fs::create_dir_all(&db_dir).ok();
            let repo = match ChannelRepo::open(
                &db_dir.join("channel.db"),
                &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("migrations"),
            ) {
                Ok(repo) => Arc::new(repo),
                Err(error) => {
                    tracing::error!(%error, "打开渠道数据库失败；渠道已禁用");
                    return Ok(());
                }
            };
            app.manage(repo.clone());

            let dispatcher = Arc::new(ChannelDispatcher::new());
            app.manage(dispatcher.clone());

            let sidecar_client = app.state::<AppState>().lifecycle.client().clone();
            let reply = Arc::new(channels::reply::ReplyCoordinator::new(sidecar_client));

            let coordinator = Arc::new(ChannelCoordinator::new(
                repo,
                dispatcher.clone(),
                reply,
                app.handle().clone(),
            ));
            app.manage(coordinator.clone());

            // 注册闲鱼协议并绑定入站监听器。
            let xianyu = Arc::new(XianyuChannel::new());
            xianyu.set_inbound_listener(coordinator.clone());
            dispatcher.register(xianyu);

            let lifecycle = app.state::<AppState>().lifecycle.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = lifecycle.ensure_running().await {
                    tracing::error!(%error, "侧车启动失败");
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            agent_ping,
            ai_config_get,
            ai_config_set,
            ai_test_api_key,
            channel_state_get,
            channel_state_set,
            channel_connect,
            channel_disconnect,
            channel_send,
            channel_open_site,
            channel_close_site,
            channel_qr_start,
            channel_qr_check,
            channel_qr_cancel,
            license_status,
            license_machine_code,
            license_activate,
            log_clear,
            log_recent
        ])
        .build(context)?
        .run(move |app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                let lifecycle = app_handle.state::<AppState>().lifecycle.clone();
                tauri::async_runtime::block_on(async move {
                    if let Err(error) = lifecycle.stop().await {
                        tracing::error!(%error, "侧车关闭失败");
                    }
                });
            }
        });

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> tauri::Result<()> {
    launch(tauri::generate_context!())
}
