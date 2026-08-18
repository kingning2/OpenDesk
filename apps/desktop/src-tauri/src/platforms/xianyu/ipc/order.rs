//! 订单管理 Tauri commands — 订单查询/状态/发货/评价联动。
//!
//! 壳层组合：`InMemoryOrderStore` → `app::order::OrderService`。

use crate::platforms::xianyu::persist::InMemoryOrderStore;
use app::order::{DeliveryInfoUpdate, Order, OrderService, OrderStatus};
use common;
use serde::Deserialize;
use std::sync::Arc;
use tauri::State;

/// 订单服务句柄（setup 时注册到 Tauri 状态）。
pub struct OrderHandle {
    pub store: Arc<InMemoryOrderStore>,
}

#[derive(Debug, Deserialize)]
pub struct OrderListRequest {
    pub owner_id: i64,
    pub page: u32,
    pub page_size: u32,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub keyword: String,
}

#[derive(Debug, Deserialize)]
pub struct OrderStatusRequest {
    pub order_no: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct OrderDeliveryRequest {
    pub order_no: String,
    pub status: String,
    pub delivery_method: String,
    pub delivery_content: Option<String>,
}

#[tauri::command]
pub fn order_list(
    state: State<'_, OrderHandle>,
    request: OrderListRequest,
) -> common::OpenDeskResult<(Vec<Order>, u32)> {
    let service = OrderService::new(state.store.as_ref());
    let status = request.status.as_deref().map(OrderStatus::from_str);
    service
        .list(
            request.owner_id,
            request.page,
            request.page_size,
            status,
            &request.keyword,
        )
        .map_err(common::OpenDeskError::wrap)
}

#[tauri::command]
pub fn order_get(
    state: State<'_, OrderHandle>,
    owner_id: i64,
    order_no: String,
) -> common::OpenDeskResult<Option<Order>> {
    let service = OrderService::new(state.store.as_ref());
    service
        .get_order_by_no(owner_id, &order_no)
        .map_err(common::OpenDeskError::wrap)
}

#[tauri::command]
pub fn order_update_status(
    state: State<'_, OrderHandle>,
    request: OrderStatusRequest,
) -> common::OpenDeskResult<bool> {
    let service = OrderService::new(state.store.as_ref());
    let status = OrderStatus::from_str(&request.status);
    service
        .update_status(&request.order_no, status)
        .map_err(common::OpenDeskError::wrap)
}

#[tauri::command]
pub fn order_update_delivery(
    state: State<'_, OrderHandle>,
    request: OrderDeliveryRequest,
) -> common::OpenDeskResult<bool> {
    let service = OrderService::new(state.store.as_ref());
    let update = DeliveryInfoUpdate {
        status: OrderStatus::from_str(&request.status),
        delivery_method: app::order::DeliveryMethod::from_str(&request.delivery_method),
        delivery_content: request.delivery_content.clone(),
        buyer_fish_nick: None,
    };
    service
        .update_delivery_info(&request.order_no, update)
        .map_err(common::OpenDeskError::wrap)
}

#[tauri::command]
pub fn order_create(state: State<'_, OrderHandle>, order: Order) -> common::OpenDeskResult<Order> {
    let service = OrderService::new(state.store.as_ref());
    service.create(&order).map_err(common::OpenDeskError::wrap)
}

#[tauri::command]
pub fn order_delete(
    state: State<'_, OrderHandle>,
    owner_id: i64,
    order_id: i64,
) -> common::OpenDeskResult<bool> {
    let service = OrderService::new(state.store.as_ref());
    service
        .delete(owner_id, order_id)
        .map_err(common::OpenDeskError::wrap)
}
