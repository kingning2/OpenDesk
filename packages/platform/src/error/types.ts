/**
 * 错误生命周期类型定义。
 *
 * 定义三类错误与统一错误载体：
 * - `network`：网络错误（sidecar 后端不可达 / HTTP 传输失败 / 健康检查失败）
 * - `ipc`：IPC 错误（Tauri invoke 拒绝 / 业务响应 code ≠ 200）
 * - `code`：代码错误（JS 未捕获异常）
 *
 * @author Xiaoman
 * @created 2026-08-18
 */

/** 错误分类。 */
export type ErrorKind = "network" | "ipc" | "code";

/** 统一错误载体。 */
export interface DeskError {
  /** 错误分类。 */
  kind: ErrorKind;
  /** 可读错误消息。 */
  message: string;
  /** 产生错误的 Tauri command 名（IPC 路径专属）。 */
  command?: string;
  /** 业务状态码（IPC 响应 code ≠ 200 时存在）。 */
  code?: number;
  /** 原始错误。 */
  cause?: unknown;
}

/**
 * IPC 错误 — Tauri invoke 拒绝或业务 code ≠ 200 时的包装。
 *
 * 通过 `instanceof` 区分 IPC 错误与普通代码错误，供全局处理器分类。
 *
 * @author Xiaoman
 * @created 2026-08-18
 */
export class IpcError extends Error {
  readonly kind: ErrorKind;
  readonly command?: string;
  readonly code?: number;
  readonly cause?: unknown;

  constructor(init: DeskError) {
    super(init.message);
    this.name = "IpcError";
    this.kind = init.kind;
    this.command = init.command;
    this.code = init.code;
    this.cause = init.cause;
  }
}
