/**
 * 发布地址库 IPC 封装 — 全局随机地址池 + 个人地址库 CRUD。
 *
 * 后端：壳层 `commands/address.rs`（InMemoryAddressStore + app::publish::AddressService）。
 *
 * @author agent
 * @created 2026-08-13
 */

import { call } from "./invoke";

/** 地址类型（与 Rust `AddressType` 对齐）。 */
export type AddressType = "global" | "personal";

/** 发布地址（与 Rust `PublishAddress` 对齐核心字段）。 */
export interface PublishAddress {
  id: number;
  owner_id: number;
  address_type: AddressType;
  address: string;
  name: string;
  search_keyword: string;
  expected_text: string | null;
  weight: number;
  sort_order: number;
  is_enabled: boolean;
  use_count: number;
  remark: string | null;
  created_at: string | null;
  updated_at: string | null;
}

const OWNER_ID = 1; // 桌面单用户；多用户时由登录态注入

/** 分页查询地址。 */
export function addressList(query: {
  page: number;
  page_size: number;
  keyword?: string;
  address_type?: AddressType;
}): Promise<[PublishAddress[], number]> {
  return call<[PublishAddress[], number]>("address_list", {
    request: {
      owner_id: OWNER_ID,
      page: query.page,
      page_size: query.page_size,
      keyword: query.keyword ?? "",
      address_type: query.address_type ?? "",
    },
  });
}

/** 新建地址。 */
export function addressCreate(
  address: Omit<PublishAddress, "id" | "owner_id">,
): Promise<PublishAddress> {
  return call<PublishAddress>("address_create", {
    ownerId: OWNER_ID,
    address: { ...address, id: 0, owner_id: OWNER_ID },
  });
}

/** 更新地址。 */
export function addressUpdate(address: PublishAddress): Promise<void> {
  return call<void>("address_update", { ownerId: OWNER_ID, address });
}

/** 删除地址。 */
export function addressDelete(addressId: number): Promise<void> {
  return call<void>("address_delete", {
    request: { owner_id: OWNER_ID, address_id: addressId },
  });
}

/** 批量删除地址（返回实际删除数量）。 */
export function addressBatchDelete(addressIds: number[]): Promise<number> {
  return call<number>("address_batch_delete", {
    request: { owner_id: OWNER_ID, address_ids: addressIds },
  });
}
