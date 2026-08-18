//! 消息通知 Tauri commands — 通知渠道 CRUD + 账号×渠道绑定。

use crate::platforms::xianyu::persist::InMemoryNotificationStore;
use crate::shared::ipc::IpcResponse;
use app::notification::{MessageNotification, NotificationChannel, NotificationService};
use common;
use serde::Deserialize;
use std::sync::Arc;
use tauri::State;

/// 通知服务句柄（setup 时注册到 Tauri 状态）。
pub struct NotificationHandle {
    pub store: Arc<InMemoryNotificationStore>,
}

#[derive(Debug, Deserialize)]
pub struct ChannelActionRequest {
    pub owner_id: i64,
    pub channel_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct ChannelEnabledRequest {
    pub owner_id: i64,
    pub channel_id: i64,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct NotificationSetRequest {
    pub owner_id: i64,
    pub account_id: String,
    pub channel_id: i64,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct NotificationDeleteRequest {
    pub owner_id: i64,
    pub notification_id: i64,
}

#[tauri::command]
pub fn notification_channel_list(
    state: State<'_, NotificationHandle>,
    owner_id: i64,
) -> common::OpenDeskResult<IpcResponse<Vec<NotificationChannel>>> {
    let service = NotificationService::new(state.store.as_ref());
    let result = service.list_channels(owner_id)?;
    Ok(IpcResponse::ok(result))
}

#[tauri::command]
pub fn notification_channel_create(
    state: State<'_, NotificationHandle>,
    owner_id: i64,
    channel: NotificationChannel,
) -> common::OpenDeskResult<IpcResponse<NotificationChannel>> {
    let service = NotificationService::new(state.store.as_ref());
    let result = service
        .create_channel(owner_id, channel)
        .map_err(common::OpenDeskError::wrap)?;
    Ok(IpcResponse::ok(result))
}

#[tauri::command]
pub fn notification_channel_update(
    state: State<'_, NotificationHandle>,
    owner_id: i64,
    channel: NotificationChannel,
) -> common::OpenDeskResult<IpcResponse<()>> {
    let service = NotificationService::new(state.store.as_ref());
    service
        .update_channel(owner_id, channel)
        .map_err(common::OpenDeskError::wrap)?;
    Ok(IpcResponse::ok(()))
}

#[tauri::command]
pub fn notification_channel_set_enabled(
    state: State<'_, NotificationHandle>,
    request: ChannelEnabledRequest,
) -> common::OpenDeskResult<IpcResponse<()>> {
    let service = NotificationService::new(state.store.as_ref());
    service.set_channel_enabled(request.owner_id, request.channel_id, request.enabled)?;
    Ok(IpcResponse::ok(()))
}

#[tauri::command]
pub fn notification_channel_test(
    state: State<'_, NotificationHandle>,
    request: ChannelActionRequest,
) -> common::OpenDeskResult<IpcResponse<String>> {
    let service = NotificationService::new(state.store.as_ref());
    let result = service.test_channel(request.owner_id, request.channel_id)?;
    Ok(IpcResponse::ok(result))
}

#[tauri::command]
pub fn notification_channel_delete(
    state: State<'_, NotificationHandle>,
    request: ChannelActionRequest,
) -> common::OpenDeskResult<IpcResponse<()>> {
    let service = NotificationService::new(state.store.as_ref());
    service.delete_channel(request.owner_id, request.channel_id)?;
    Ok(IpcResponse::ok(()))
}

#[tauri::command]
pub fn notification_list(
    state: State<'_, NotificationHandle>,
    owner_id: i64,
) -> common::OpenDeskResult<IpcResponse<Vec<MessageNotification>>> {
    let service = NotificationService::new(state.store.as_ref());
    let result = service.list_notifications(owner_id)?;
    Ok(IpcResponse::ok(result))
}

#[tauri::command]
pub fn notification_set(
    state: State<'_, NotificationHandle>,
    request: NotificationSetRequest,
) -> common::OpenDeskResult<IpcResponse<MessageNotification>> {
    let service = NotificationService::new(state.store.as_ref());
    let result = service.set_notification(
        request.owner_id,
        &request.account_id,
        request.channel_id,
        request.enabled,
    )?;
    Ok(IpcResponse::ok(result))
}

#[tauri::command]
pub fn notification_delete(
    state: State<'_, NotificationHandle>,
    request: NotificationDeleteRequest,
) -> common::OpenDeskResult<IpcResponse<()>> {
    let service = NotificationService::new(state.store.as_ref());
    service.delete_notification(request.owner_id, request.notification_id)?;
    Ok(IpcResponse::ok(()))
}
