/**
 * 商品发布素材库 IPC 封装 — 素材 CRUD + 批量删除。
 *
 * 后端：壳层 `commands/publish_material.rs`（InMemoryPublishMaterialStore + app::publish::PublishMaterialService）。
 *
 * @author agent
 * @created 2026-08-13
 */

import { call } from "./invoke";

/** 发布素材（与 Rust `PublishMaterial` 对齐核心字段）。 */
export interface PublishMaterial {
  id: number;
  owner_id: number;
  title: string;
  description: string;
  price: number;
  original_price: number | null;
  category: string | null;
  platform_category_id: string | null;
  platform_category_name: string | null;
  images: string;
  condition: string;
  quantity: number;
  delivery_method: string;
  shipping_method: string;
  postage: number;
  brand: string | null;
  remark: string | null;
  created_at: string | null;
  updated_at: string | null;
}

/** 素材查询条件。 */
export interface PublishMaterialQuery {
  page: number;
  page_size: number;
  keyword?: string;
  category?: string;
  condition?: string;
  platform_category_id?: string;
}

const OWNER_ID = 1; // 桌面单用户；多用户时由登录态注入

/** 分页查询素材。 */
export function publishMaterialList(
  query: PublishMaterialQuery,
): Promise<[PublishMaterial[], number]> {
  return call<[PublishMaterial[], number]>("publish_material_list", {
    request: {
      owner_id: OWNER_ID,
      page: query.page,
      page_size: query.page_size,
      keyword: query.keyword ?? "",
      category: query.category ?? "",
      condition: query.condition ?? "",
      platform_category_id: query.platform_category_id ?? "",
    },
  });
}

/** 新建素材。 */
export function publishMaterialCreate(
  material: Omit<PublishMaterial, "id" | "owner_id">,
): Promise<PublishMaterial> {
  return call<PublishMaterial>("publish_material_create", {
    ownerId: OWNER_ID,
    material: { ...material, id: 0, owner_id: OWNER_ID },
  });
}

/** 更新素材。 */
export function publishMaterialUpdate(material: PublishMaterial): Promise<void> {
  return call<void>("publish_material_update", { ownerId: OWNER_ID, material });
}

/** 删除素材。 */
export function publishMaterialDelete(materialId: number): Promise<void> {
  return call<void>("publish_material_delete", {
    request: { owner_id: OWNER_ID, material_id: materialId },
  });
}

/** 批量删除素材（返回实际删除数量）。 */
export function publishMaterialBatchDelete(materialIds: number[]): Promise<number> {
  return call<number>("publish_material_batch_delete", {
    request: { owner_id: OWNER_ID, material_ids: materialIds },
  });
}
