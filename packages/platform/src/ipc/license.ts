/**
 * License IPC 类型与调用封装（无状态）。
 *
 * 负责：
 * - 定义与 Rust DTO 对齐的前端类型
 * - 封装 license_status / license_machine_code / license_activate
 *
 * @author coisini
 * @created 2026-07-16
 */

import { invoke } from "@tauri-apps/api/core";

/**
 * 授权状态（与 Rust `LicenseStatus` camelCase 对齐）。
 *
 * @author coisini
 * @created 2026-07-16
 */
export interface LicenseStatus {
  /** 是否启用授权闸门。 */
  gateEnabled: boolean;
  /** 是否已通过校验。 */
  activated: boolean;
  /** 失败或未激活原因。 */
  reason?: string | null;
  /** 本机设备码。 */
  machineCode?: string | null;
  /** 过期时间（Unix 秒）。 */
  expiresAt?: number | null;
  /** 产品名。 */
  product?: string | null;
}

/**
 * 激活请求；`token` 与 `keyBytesBase64` 必须恰好提供其一。
 *
 * @author coisini
 * @created 2026-07-16
 */
export interface LicenseActivateRequest {
  /** 粘贴的激活 token。 */
  token?: string | null;
  /** `.key` 文件字节的标准 Base64。 */
  keyBytesBase64?: string | null;
}

/**
 * 查询当前授权状态。
 *
 * @author coisini
 * @created 2026-07-16
 *
 * @returns 授权状态 DTO
 */
export function licenseStatus(): Promise<LicenseStatus> {
  return invoke<LicenseStatus>("license_status");
}

/**
 * 读取本机设备码。
 *
 * @author coisini
 * @created 2026-07-16
 *
 * @returns 设备码字符串
 */
export function licenseMachineCode(): Promise<string> {
  return invoke<string>("license_machine_code");
}

/**
 * 提交激活请求。
 *
 * @author coisini
 * @created 2026-07-16
 *
 * @param request - token 或 key Base64
 * @returns 激活后的授权状态
 */
export function licenseActivate(
  request: LicenseActivateRequest,
): Promise<LicenseStatus> {
  return invoke<LicenseStatus>("license_activate", { request });
}
