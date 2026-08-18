/**
 * 账号管理 IPC 封装 — 多账号 CRUD + 状态切换。
 *
 * 后端：壳层 `commands/account.rs`（InMemoryAccountStore + app::account::AccountService）。
 *
 * @author agent
 * @created 2026-08-13
 */

import { callRequest, type IpcResponse } from "./invoke";

/** 账号状态（与 Rust `AccountStatus` 对齐）。 */
export type AccountStatus = "active" | "disabled";

/** 账号（与 Rust `app::account::XianyuAccount` 对齐的核心字段）。 */
export interface XianyuAccount {
  id: number;
  owner_id: number;
  /** 账号标识（全局唯一）。 */
  account_id: string;
  display_name: string;
  login_id: string;
  login_password: string;
  unb: string;
  cookie: string;
  /** qr / password。 */
  login_method: string;
  status: AccountStatus;
  remark: string;
  pause_duration_minutes: number;
}

/** 账号更新补丁（缺省字段不更新）。 */
export interface AccountUpdate {
  display_name?: string;
  remark?: string;
  status?: AccountStatus;
  login_id?: string;
  login_password?: string;
}

/** 查询账号列表。 */
export function accountList(ownerId: number): Promise<XianyuAccount[]> {
  return callRequest<XianyuAccount[]>("account_list", { ownerId }).then((response) => response.data);
}

/** 新建账号。 */
export function accountCreate(
  ownerId: number,
  account: XianyuAccount,
): Promise<XianyuAccount> {
  return callRequest<XianyuAccount>("account_create", { ownerId, account }).then(
    (response) => response.data,
  );
}

/** 更新账号（部分字段）。 */
export function accountUpdate(
  ownerId: number,
  accountId: string,
  patch: AccountUpdate,
): Promise<XianyuAccount> {
  return callRequest<XianyuAccount>("account_update", { ownerId, accountId, patch }).then(
    (response) => response.data,
  );
}

/** 切换账号启用状态。 */
export function accountSetStatus(
  ownerId: number,
  accountId: string,
  status: AccountStatus,
): Promise<void> {
  return callRequest<void>("account_set_status", {
    request: { owner_id: ownerId, account_id: accountId, status },
  }).then(() => undefined);
}

/** 删除账号。 */
export function accountDelete(ownerId: number, accountId: string): Promise<void> {
  return callRequest<void>("account_delete", {
    request: { owner_id: ownerId, account_id: accountId },
  }).then(() => undefined);
}

// ========== 业务账号扫码登录（复用 sidecar 扫码能力，成功后自动创建/更新账号） ==========

/** 扫码启动响应（与 Rust `ChannelIpcQrStartResponse` 对齐）。 */
export interface AccountQrStartResult {
  ok: boolean;
  status: string;
  session_id: string | null;
  qr_base64: string | null;
  detail: string | null;
}

/** 扫码状态轮询响应（与 Rust `ChannelIpcQrCheckResponse` 对齐）。 */
export interface AccountQrCheckResult {
  ok: boolean;
  status: string;
  session_id: string | null;
  cookies: unknown[] | null;
  detail: string | null;
  qr_base64: string | null;
}

/** 启动业务账号扫码登录。 */
export function accountQrStart(name?: string): Promise<AccountQrStartResult> {
  return callRequest<AccountQrStartResult>("account_qr_start", {
    request: { name: name ?? null },
  }).then((response) => response.data);
}

/** 轮询扫码状态（成功后账号已自动创建/更新）。 */
export function accountQrCheck(sessionId: string): Promise<AccountQrCheckResult> {
  return callRequest<AccountQrCheckResult>("account_qr_check", {
    request: { session_id: sessionId },
  }).then((response) => response.data);
}

/** 取消业务账号扫码登录。 */
export function accountQrCancel(sessionId: string): Promise<void> {
  return callRequest<void>("account_qr_cancel", {
    request: { session_id: sessionId },
  }).then(() => undefined);
}

// ========== 业务账号密码登录（Playwright 真实浏览器上下文登录，成功后自动创建/更新账号） ==========

/** 账号密码登录响应。 */
export interface AccountPasswordLoginResult {
  ok: boolean;
  status: string;
  account_id: string | null;
  detail: string | null;
}

/**
 * 使用账号密码登录业务账号。
 *
 * @author Xiaoman
 * @created 2026-08-18
 *
 * @param loginId - 登录账号（手机号/用户名/邮箱）
 * @param password - 登录密码
 * @param name - 可选展示名
 * @returns 登录结果；成功时返回自动创建/更新后的账号标识
 */
export function accountPasswordLogin(
  loginId: string,
  password: string,
  name?: string,
): Promise<IpcResponse<AccountPasswordLoginResult>> {
  return callRequest<AccountPasswordLoginResult>("account_password_login", {
    request: {
      login_id: loginId,
      password,
      name: name ?? null,
    },
  });
}

// ========== 业务账号渠道连接（扫码后自动连接 / 手动断开连接） ==========

/** 连接业务账号（建立渠道 websocket 设备监听）。 */
export function accountConnect(ownerId: number, accountId: string): Promise<string> {
  return callRequest<string>("account_connect", {
    request: { owner_id: ownerId, account_id: accountId },
  }).then((response) => response.data);
}

/** 断开业务账号的渠道连接。 */
export function accountDisconnect(ownerId: number, accountId: string): Promise<void> {
  return callRequest<void>("account_disconnect", {
    request: { owner_id: ownerId, account_id: accountId },
  }).then(() => undefined);
}

/** 查询业务账号的渠道连接状态（connected / connecting / disconnected / error）。 */
export function accountConnectionState(ownerId: number, accountId: string): Promise<string> {
  return callRequest<string>("account_connection_state", {
    request: { owner_id: ownerId, account_id: accountId },
  }).then((response) => response.data);
}
