import { invoke } from "@tauri-apps/api/core";

/** 单条运行日志（Rust / Python 统一）。 */
export interface LogEntry {
  /** Unix 毫秒时间戳。 */
  ts: number;
  /** 级别：TRACE / DEBUG / INFO / WARN / ERROR。 */
  level: string;
  /** 来源：rust | python。 */
  source: string;
  /** 目标模块。 */
  target: string;
  /** 格式化消息。 */
  message: string;
}

/** 读取最近日志（时间正序，旧 → 新）。 */
export function logRecent(limit = 500): Promise<LogEntry[]> {
  return invoke<LogEntry[]>("log_recent", { limit });
}

/** 清空日志缓冲。 */
export function logClear(): Promise<void> {
  return invoke<void>("log_clear");
}
