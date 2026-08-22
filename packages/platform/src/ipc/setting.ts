/**
 * 用户设置 IPC 封装 — 用户级键值配置存取。
 *
 * 后端：壳层 `commands/setting.rs`（InMemoryUserSettingStore + app::setting::UserSettingService）。
 *
 * @author agent
 * @created 2026-08-13
 */

import { call } from "./invoke";

const OWNER_ID = 1; // 桌面单用户；多用户时由登录态注入

/** 读取单键设置。 */
export function userSettingGet(key: string): Promise<string | null> {
  return call<string | null>("user_setting_get", {
    request: { owner_id: OWNER_ID, key },
  });
}

/** 写入单键设置（空值删除）。 */
export function userSettingSet(key: string, value: string): Promise<void> {
  return call<void>("user_setting_set", {
    request: { owner_id: OWNER_ID, key, value },
  });
}
