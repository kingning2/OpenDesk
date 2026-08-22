//! 平台运行时装配 — 风控、业务 Handle、渠道协议注册。
//!
//! 条件编译收敛在本模块；`lib.rs` setup 仅调用 [`register_platform`]。
//!
//! 作者：Xiaoman
//! 创建时间：2026-08-22

use crate::shared::channel::coordinator::ChannelCoordinator;
use crate::shared::channel::dispatcher::ChannelDispatcher;
use crate::shared::channel::risk_handler::RiskHandler;
use common::events::EventSink;
use common::DingDaResult;
use std::sync::Arc;

#[cfg(platform_xianyu)]
use tauri::Manager;

/// 按编译期平台装配风控处理器；1688 等无平台风控时返回 `None`。
pub fn build_risk_handler(
    app: &tauri::AppHandle,
    dispatcher: &Arc<ChannelDispatcher>,
    event_sink: Arc<dyn EventSink>,
) -> Option<Arc<dyn RiskHandler>> {
    #[cfg(platform_xianyu)]
    {
        Some(crate::platforms::xianyu::bootstrap::build_risk_handler(
            app, dispatcher, event_sink,
        ))
    }
    #[cfg(not(platform_xianyu))]
    {
        let _ = (app, dispatcher, event_sink);
        None
    }
}

/// 注册平台专属业务 Handle 与渠道协议（闲鱼 WS 等）。
pub fn register_platform(
    app: &tauri::AppHandle,
    dispatcher: &Arc<ChannelDispatcher>,
    coordinator: &Arc<ChannelCoordinator>,
) -> DingDaResult<()> {
    #[cfg(platform_xianyu)]
    {
        crate::platforms::xianyu::bootstrap::register_business(app)?;
        let account_store = app
            .try_state::<crate::platforms::core::account::AccountHandle>()
            .map(|handle| handle.store.clone());
        crate::platforms::xianyu::bootstrap::register_active_platform(
            dispatcher,
            coordinator,
            account_store,
        );
    }
    #[cfg(not(platform_xianyu))]
    {
        let _ = (app, dispatcher, coordinator);
    }
    Ok(())
}
