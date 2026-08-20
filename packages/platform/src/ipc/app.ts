/**
 * 应用版本 IPC 封装。
 *
 * @author Xiaoman
 * @created 2026-08-20
 */

import { call } from "./invoke";

/**
 * 读取当前应用版本。
 *
 * @author Xiaoman
 * @created 2026-08-20
 *
 * @returns 语义化版本字符串
 */
export function appVersion(): Promise<string> {
  return call<string>("app_version");
}
