/**
 * 日志面板 IPC 封装。
 *
 * @author Xiaoman
 * @created 2026-08-13
 */

import { call } from "./invoke";

/**
 * 单条运行日志（Rust / Python / React 统一）。
 *
 * @author Xiaoman
 * @created 2026-08-13
 */
export interface LogEntry {
  /** Unix 毫秒时间戳。 */
  ts: number;
  /** 级别：TRACE / DEBUG / INFO / WARN / ERROR。 */
  level: string;
  /** 来源：rust | python | react。 */
  source: string;
  /** 目标模块。 */
  target: string;
  /** 格式化消息。 */
  message: string;
}

/**
 * 读取最近日志（时间正序，旧 → 新）。
 *
 * @author Xiaoman
 * @created 2026-08-13
 *
 * @param limit - 最多返回条数
 * @returns 日志条目列表
 */
export function logRecent(limit = 500): Promise<LogEntry[]> {
  return call<LogEntry[]>("log_recent", { limit });
}

/**
 * 清空日志缓冲。
 *
 * @author Xiaoman
 * @created 2026-08-13
 */
export function logClear(): Promise<void> {
  return call<void>("log_clear");
}
