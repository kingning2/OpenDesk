/**
 * 订单管理 IPC 封装 — 查询/状态/发货/评价联动。
 *
 * 后端：壳层 `commands/order.rs`（InMemoryOrderStore + app::order::OrderService）。
 *
 * @author agent
 * @created 2026-08-13
 */

import { call } from "./invoke";

/** 订单状态（与 Rust `OrderStatus` 对齐）。 */
export type OrderStatus =
  | "pending"
  | "paid"
  | "shipped"
  | "completed"
  | "closed"
  | "refunded"
  | "unknown";

/** 订单（与 Rust `app::order::Order` 对齐的核心字段）。 */
export interface Order {
  id: number;
  owner_id: number;
  order_no: string;
  status: OrderStatus;
  buyer_id: string;
  item_id: string;
  item_title: string;
  quantity: number;
  amount: number;
  account_id: string;
  is_rated: boolean;
  is_red_flower: boolean;
}

/** 订单列表查询入参。 */
export interface OrderListRequest {
  owner_id: number;
  page: number;
  page_size: number;
  status?: OrderStatus;
  keyword?: string;
}

/** 查询订单列表（返回 [列表, 总数]）。 */
export function orderList(request: OrderListRequest): Promise<[Order[], number]> {
  return call<[Order[], number]>("order_list", { request });
}

/** 按订单号查询（归属校验）。 */
export function orderGet(ownerId: number, orderNo: string): Promise<Order | null> {
  return call<Order | null>("order_get", { ownerId, orderNo });
}

/** 更新订单状态。 */
export function orderUpdateStatus(orderNo: string, status: OrderStatus): Promise<boolean> {
  return call<boolean>("order_update_status", {
    request: { order_no: orderNo, status },
  });
}

/** 更新发货信息（状态/方式/内容）。 */
export function orderUpdateDelivery(
  orderNo: string,
  status: OrderStatus,
  deliveryMethod: string,
  deliveryContent?: string,
): Promise<boolean> {
  return call<boolean>("order_update_delivery", {
    request: {
      order_no: orderNo,
      status,
      delivery_method: deliveryMethod,
      delivery_content: deliveryContent,
    },
  });
}

/** 新建订单。 */
export function orderCreate(order: Order): Promise<Order> {
  return call<Order>("order_create", { order });
}

/** 删除订单（归属校验）。 */
export function orderDelete(ownerId: number, orderId: number): Promise<boolean> {
  return call<boolean>("order_delete", { ownerId, orderId });
}
