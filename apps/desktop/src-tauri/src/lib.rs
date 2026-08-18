//! Tauri shell：组装 AppState、注册 IPC commands、启动 sidecar。
//!
//! 目录约定：
//! - [`shared`] — 三平台共用（IPC、渠道编排、Agent、License、日志）
//! - [`platforms`] — 编译期平台壳层（当前仅闲鱼，`platform_xianyu` cfg 下链接）
//!
//! 根目录仅保留 `lib.rs` / `main.rs`。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-16

mod shared;

#[cfg(platform_xianyu)]
mod platforms;

// `#[timed]` 展开为 `crate::timing`；命令层历史路径 `crate::agent` / `crate::state` 等亦走此 re-export。
pub use shared::{agent, ai_config, logging, state, timing};

use adapter::agent_sidecar::RuntimeAgentSidecar;
use kernel::event::{EventBus, InMemoryEventBus};
use runtime::sidecar::lifecycle::{SidecarConfig, SidecarLifecycle};
use shared::channel::coordinator::ChannelCoordinator;
use shared::channel::dispatcher::ChannelDispatcher;
use shared::channel::ChannelRepo;
use shared::ipc::{
    agent_ping, ai_config_get, ai_config_set, ai_test_api_key, channel_connect, channel_disconnect,
    channel_qr_cancel, channel_qr_check, channel_qr_start, channel_send, channel_state_get,
    channel_state_set, license_activate, license_machine_code, license_status, log_clear,
    log_recent, log_write, platform_descriptors,
};
use shared::lifecycle::{on_exit, on_setup};
use shared::{build_license_gate, init_tracing, platform_initialization_script, AppState};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;

#[cfg(platform_xianyu)]
use platforms::xianyu::bootstrap::register_active_platform;
#[cfg(platform_xianyu)]
use platforms::xianyu::ipc::{
    account_connect, account_connection_state, account_create, account_delete, account_disconnect,
    account_list, account_password_login, account_qr_cancel, account_qr_check, account_qr_start,
    account_set_status, account_update, address_batch_delete, address_create, address_delete,
    address_list, address_update, auto_reply_log_list, blacklist_delete, blacklist_personal_create,
    blacklist_personal_list, blacklist_platform_list, blacklist_set_enabled, card_create,
    card_delete, card_list, card_set_enabled, card_update, channel_close_site, channel_open_site,
    dashboard_stats, feedback_create, feedback_delete, feedback_list, filter_create, filter_delete,
    filter_list, filter_set_enabled, filter_update, item_get, item_list, item_update, keyword_add,
    keyword_delete, keyword_list, keyword_replace, notification_channel_create,
    notification_channel_delete, notification_channel_list, notification_channel_set_enabled,
    notification_channel_test, notification_channel_update, notification_delete, notification_list,
    notification_set, order_create, order_delete, order_get, order_list, order_update_delivery,
    order_update_status, publish_batch_status, publish_batch_submit, publish_capability,
    publish_log_clear, publish_log_list, publish_material_batch_delete, publish_material_create,
    publish_material_delete, publish_material_list, publish_material_update, publish_single,
    rate_buyer, rate_feedback_resolve, risk_config_get, risk_config_set, risk_log_clear,
    risk_log_clear_processing, risk_log_list, risk_log_today_rate, user_setting_get,
    user_setting_set, user_settings_get_all,
};

/// 按编译期平台组装 IPC handler 列表。
macro_rules! base_invoke_handler {
    () => {{
        #[cfg(platform_xianyu)]
        {
            tauri::generate_handler![
                agent_ping,
                ai_config_get,
                ai_config_set,
                ai_test_api_key,
                address_list,
                address_create,
                address_update,
                address_delete,
                address_batch_delete,
                auto_reply_log_list,
                account_list,
                account_create,
                account_update,
                account_set_status,
                account_delete,
                account_password_login,
                account_qr_start,
                account_qr_check,
                account_qr_cancel,
                account_connect,
                account_disconnect,
                account_connection_state,
                order_list,
                order_get,
                order_update_status,
                order_update_delivery,
                order_create,
                order_delete,
                keyword_list,
                keyword_replace,
                keyword_add,
                keyword_delete,
                item_list,
                item_get,
                item_update,
                card_list,
                card_create,
                card_update,
                card_set_enabled,
                card_delete,
                blacklist_personal_list,
                blacklist_platform_list,
                blacklist_personal_create,
                blacklist_set_enabled,
                blacklist_delete,
                filter_list,
                filter_create,
                filter_update,
                filter_set_enabled,
                filter_delete,
                feedback_list,
                feedback_create,
                feedback_delete,
                notification_channel_list,
                notification_channel_create,
                notification_channel_update,
                notification_channel_set_enabled,
                notification_channel_test,
                notification_channel_delete,
                notification_list,
                notification_set,
                notification_delete,
                risk_log_list,
                risk_log_today_rate,
                risk_log_clear,
                risk_log_clear_processing,
                risk_config_get,
                risk_config_set,
                user_setting_get,
                user_setting_set,
                user_settings_get_all,
                publish_material_list,
                publish_material_create,
                publish_material_update,
                publish_material_delete,
                publish_material_batch_delete,
                publish_batch_submit,
                publish_batch_status,
                publish_capability,
                publish_single,
                publish_log_list,
                publish_log_clear,
                dashboard_stats,
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
                platform_descriptors,
                rate_buyer,
                rate_feedback_resolve,
                log_clear,
                log_recent,
                log_write
            ]
        }
        #[cfg(not(platform_xianyu))]
        {
            tauri::generate_handler![
                agent_ping,
                ai_config_get,
                ai_config_set,
                ai_test_api_key,
                channel_state_get,
                channel_state_set,
                channel_connect,
                channel_disconnect,
                channel_send,
                channel_qr_start,
                channel_qr_check,
                channel_qr_cancel,
                license_status,
                license_machine_code,
                license_activate,
                platform_descriptors,
                log_clear,
                log_recent,
                log_write
            ]
        }
    }};
}

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
        .append_invoke_initialization_script(platform_initialization_script())
        .manage(app_state)
        .setup(move |app| {
            let config_dir = match app.path().app_config_dir() {
                Ok(dir) => dir,
                Err(error) => {
                    tracing::error!(%error, "解析应用配置目录失败；AI 配置已禁用");
                    PathBuf::from(".")
                }
            };
            app.manage(Arc::new(shared::ai_config::AiConfigStore::new(
                config_dir.clone(),
            )));

            #[cfg(platform_xianyu)]
            if let Err(error) =
                platforms::xianyu::bootstrap::register_business(app.handle(), &config_dir)
            {
                tracing::error!(%error, "打开业务数据库失败；闲鱼业务已禁用");
                return Ok(());
            }

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

            let auto_reply = shared::auto_reply::AutoReplyHandle::new();

            let event_sink: Arc<dyn common::events::EventSink> =
                Arc::new(shared::TauriEventSink::new(app.handle().clone()));
            let coordinator = Arc::new(ChannelCoordinator::new(
                repo,
                dispatcher.clone(),
                auto_reply,
                event_sink,
            ));
            app.manage(coordinator.clone());

            #[cfg(platform_xianyu)]
            register_active_platform(&dispatcher, &coordinator);

            let lifecycle = app.state::<AppState>().lifecycle.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = lifecycle.ensure_running().await {
                    tracing::error!(%error, "侧车启动失败");
                }
            });
            on_setup();
            Ok(())
        })
        .invoke_handler(base_invoke_handler!())
        .build(context)?
        .run(move |app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                on_exit();
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
