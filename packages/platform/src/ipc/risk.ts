/**
 * 风控日志 IPC 封装 — 日志查询 / 清空 / 滑块配置存取。
 *
 * 后端：壳层 `commands/risk.rs`（InMemoryRiskStore + app::risk::RiskService）。
 *
 * @author agent
 * @created 2026-08-13
 */

import { call } from "./invoke";

/** 风控日志条目（与 Rust `RiskLogItem` 对齐）。 */
export interface RiskLogItem {
  id: number;
  owner_id: number;
  account_id: string;
  risk_type: string;
  message: string;
  processing_result: string;
  processing_status: string;
  captcha_engine: string | null;
  call_type: string | null;
  call_user: string | null;
  error_message: string | null;
  created_at: string | null;
  updated_at: string | null;
}

/** 分页结果（与 Rust `RiskLogPage` 对齐）。 */
export interface RiskLogPage {
  data: RiskLogItem[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

/** 当日成功率（与 Rust `RiskTodaySuccessRate` 对齐）。 */
export interface RiskTodaySuccessRate {
  date: string;
  total: number;
  success: number;
  rate: number;
  local_total: number;
  local_success: number;
  local_rate: number;
  remote_total: number;
  remote_success: number;
  remote_rate: number;
  processing: number;
  local_processing: number;
  remote_processing: number;
}

/** 远程过滑块配置 + 本机滑块开关（与 Rust `RiskConfig` 对齐）。 */
export interface RiskConfig {
  remote_url: string;
  remote_secret: string;
  pass_cookies: boolean;
  block_remote_calls: boolean;
  local_weight: number;
  remote_weight: number;
  remote_processing_max: number;
  remote_cooldown_seconds: number;
  local_slider_disabled: boolean;
}

/** 日志查询条件。 */
export interface RiskLogQuery {
  page: number;
  page_size: number;
  account_id?: string;
  start_date?: string;
  end_date?: string;
  processing_status?: string;
  call_type?: string;
  call_user?: string;
}

/** 分页查询风控日志。 */
export function riskLogList(query: RiskLogQuery): Promise<RiskLogPage> {
  return call<RiskLogPage>("risk_log_list", {
    request: {
      owner_id: 1, // 桌面单用户；多用户时由登录态注入
      page: query.page,
      page_size: query.page_size,
      account_id: query.account_id ?? "",
      start_date: query.start_date ?? "",
      end_date: query.end_date ?? "",
      processing_status: query.processing_status ?? "",
      call_type: query.call_type ?? "",
      call_user: query.call_user ?? "",
    },
  });
}

/** 当日风控成功率。 */
export function riskLogTodayRate(date: string): Promise<RiskTodaySuccessRate> {
  return call<RiskTodaySuccessRate>("risk_log_today_rate", {
    request: { owner_id: 1, date },
  });
}

/** 清空风控日志（accountId 为空则全部）。 */
export function riskLogClear(accountId?: string): Promise<void> {
  return call<void>("risk_log_clear", {
    request: { owner_id: 1, account_id: accountId ?? "" },
  });
}

/** 清空处理中日志。 */
export function riskLogClearProcessing(): Promise<void> {
  return call<void>("risk_log_clear_processing", { ownerId: 1 });
}

/** 读取风控配置。 */
export function riskConfigGet(): Promise<RiskConfig> {
  return call<RiskConfig>("risk_config_get", { ownerId: 1 });
}

/** 保存风控配置。 */
export function riskConfigSet(config: RiskConfig): Promise<void> {
  return call<void>("risk_config_set", {
    request: { owner_id: 1, config },
  });
}
