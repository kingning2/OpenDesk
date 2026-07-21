import { invoke } from "@tauri-apps/api/core";
import type {
  CustomerDtoProfile,
  CustomerIpcCreateRequest,
  CustomerIpcCreateResponse,
  CustomerIpcGetRequest,
  CustomerIpcGetResponse,
  CustomerIpcListRequest,
  CustomerIpcListResponse,
  CustomerIpcUpdateRequest,
  CustomerIpcUpdateResponse,
} from "@desk/contracts";

export type CustomerProfile = CustomerDtoProfile;

/**
 * List customers with optional search and pagination.
 *
 * @author Xiaoman
 * @created 2026-07-21
 */
export async function customerList(
  input: CustomerIpcListRequest = {},
): Promise<{ items: CustomerProfile[]; total: number }> {
  const response = await invoke<CustomerIpcListResponse>("customer_list", { request: input });
  try {
    const parsed = JSON.parse(response.customers_json ?? "[]") as CustomerProfile[];
    return {
      items: Array.isArray(parsed) ? parsed : [],
      total: response.total ?? 0,
    };
  } catch {
    return { items: [], total: 0 };
  }
}

/**
 * Fetch one customer profile by id.
 *
 * @author Xiaoman
 * @created 2026-07-21
 */
export async function customerGet(id: string): Promise<CustomerProfile> {
  const request: CustomerIpcGetRequest = { id };
  const response = await invoke<CustomerIpcGetResponse>("customer_get", { request });
  return JSON.parse(response.profile_json) as CustomerProfile;
}

/**
 * Create a new customer profile.
 *
 * @author Xiaoman
 * @created 2026-07-21
 */
export async function customerCreate(
  input: CustomerIpcCreateRequest,
): Promise<CustomerProfile> {
  const response = await invoke<CustomerIpcCreateResponse>("customer_create", { request: input });
  return JSON.parse(response.profile_json) as CustomerProfile;
}

/**
 * Update an existing customer profile.
 *
 * @author Xiaoman
 * @created 2026-07-21
 */
export async function customerUpdate(
  input: CustomerIpcUpdateRequest,
): Promise<CustomerProfile> {
  const response = await invoke<CustomerIpcUpdateResponse>("customer_update", { request: input });
  return JSON.parse(response.profile_json) as CustomerProfile;
}
