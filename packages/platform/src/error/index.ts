/**
 * 错误生命周期 — 类型 / 分类 / 上报通道。
 *
 * @author Xiaoman
 * @created 2026-08-18
 */

export { classifyError, classifyKind, isClassified, stringifyError } from "./classify";
export { reportError, setErrorReporter, type ErrorReporter } from "./reporter";
export { IpcError, type DeskError, type ErrorKind } from "./types";
