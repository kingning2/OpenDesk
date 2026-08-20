/**
 * 付费功能是否已授权（前端展示用；后端 IPC 仍会做 `ensure_licensed` 校验）。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */

import type { LicenseStatus } from "@desk/platform/ipc/license";

/**
 * 判断当前是否可使用付费功能。
 *
 * @author Xiaoman
 * @created 2026-08-20
 *
 * @param status - 授权状态
 * @returns 无闸门或已激活时返回 true
 */
export function isLicensed(status: LicenseStatus | null | undefined): boolean {
  if (!status?.gateEnabled) {
    return true;
  }
  return status.activated;
}
