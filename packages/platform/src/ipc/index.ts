import { invokeIpc } from "./invoke";

export function call<T>(command: string, payload?: Record<string, unknown>) {
  return invokeIpc<T>(command, payload);
}

export {
  DIAGNOSTICS_LOG_COMMAND,
  formatInvokeError,
  InvokeError,
  invokeIpc,
  installIpcErrorReporter,
  reportFrontendError,
  reportFrontendErrorValue,
  setInvokeErrorReporter,
  writeDiagnosticsLog,
  type FrontendErrorKind,
  type FrontendErrorReport,
  type InvokeErrorReporter,
} from "./invoke";

export * from "./customer";
export * from "./chat";
export * from "./help";
export * from "./mail";
export * from "./mail-events";
export * from "./workflow";
export * from "./workflow-runtime";
export * from "./workflow-runtime-events";
export * from "./llm-settings";
export * from "./mail-integration";
